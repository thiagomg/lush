use mlua::{Lua, Value, Result};
use rustyline::{ColorMode, Config, Editor};
use crate::setup;
use crate::setup::LushContext;

use colored::Colorize;
use rustyline::history::DefaultHistory;
use crate::lush_highlighter::LushHighlighter;

pub fn run_repl() -> Result<()> {
    let lua = Lua::new();
    let ctx = LushContext {
        dir_stack: vec![],
    };
    lua.set_app_data(ctx);
    setup::set_utils(&lua)?;

    let config = Config::builder()
        .color_mode(ColorMode::Enabled)
        .check_cursor_position(true)
        .build();

    let mut rl = Editor::<LushHighlighter, DefaultHistory>::with_config(config)
        .expect("Could not create RL environment");
    rl.set_helper(Some(LushHighlighter));

    println!("{}. Press {} or type `{}` to quit.", "LuSH REPL".bold(), "Ctrl+D".bold(), "exit".bold());

    loop {
        let mut input = String::new();
        let mut line_count = 0;

        loop {
            let prompt = if line_count == 0 {
                "lush> "
            } else {
                "....> "
            };

            let readline = rl.readline(prompt);
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();

                    // Check for exit on first line only
                    if line_count == 0 && trimmed == "exit" {
                        return Ok(());
                    }

                    // Add the line to our input
                    if line_count > 0 {
                        input.push('\n');
                    }
                    input.push_str(&line);
                    line_count += 1;

                    // Check if the input is complete
                    if is_complete_statement(&input) {
                        break;
                    }
                }
                Err(_) => return Ok(()), // e.g., Ctrl+D
            }
        }

        let trimmed_input = input.trim();
        if !trimmed_input.is_empty() {
            rl.add_history_entry(trimmed_input).ok();

            match lua.load(trimmed_input).eval::<Value>() {
                Ok(Value::Nil) => {
                    if is_valid_lua_identifier(trimmed_input) {
                        println!("Nil");
                    }
                }
                Ok(result) => println!("=> {:?}", result),
                Err(err) => print_repl_error(err),
            }
        }
    }
}

fn is_valid_lua_identifier(s: &str) -> bool {
    let reserved = [
        "and", "break", "do", "else", "elseif", "end", "false", "for", "function",
        "goto", "if", "in", "local", "nil", "not", "or", "repeat", "return",
        "then", "true", "until", "while"
    ];
    let re = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    re.is_match(s) && !reserved.contains(&s)
}

fn print_repl_error(err: mlua::Error) {
    eprintln!("Error: {}", clean_lua_error(&err.to_string()).red());
}

fn is_complete_statement(input: &str) -> bool {
    // Remove string literals and comments to avoid false positives
    let cleaned = remove_strings_and_comments(input);

    // Check for balanced braces, brackets, and parentheses
    if !is_balanced(&cleaned) {
        return false;
    }

    // Check for incomplete control structures
    if has_incomplete_control_structure(&cleaned) {
        return false;
    }

    // Try to compile (but not execute) with Lua to see if it's syntactically complete
    // This is a more robust check that doesn't execute the code
    let lua = Lua::new();
    match lua.load(input).into_function() {
        Ok(_) => true,
        Err(err) => {
            let err_str = err.to_string();
            // Check if it's a syntax error indicating incomplete input
            !is_incomplete_syntax_error(&err_str)
        }
    }
}

fn remove_strings_and_comments(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Skip double-quoted string
                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '"' {
                        break;
                    }
                    if inner_ch == '\\' {
                        chars.next(); // Skip escaped character
                    }
                }
            }
            '\'' => {
                // Skip single-quoted string
                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '\'' {
                        break;
                    }
                    if inner_ch == '\\' {
                        chars.next(); // Skip escaped character
                    }
                }
            }
            '[' => {
                // Check for long string literal [[...]]
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume second [
                    let mut bracket_count = 0;
                    while let Some(inner_ch) = chars.next() {
                        if inner_ch == ']' && chars.peek() == Some(&']') {
                            chars.next(); // consume second ]
                            if bracket_count == 0 {
                                break;
                            }
                            bracket_count -= 1;
                        } else if inner_ch == '[' && chars.peek() == Some(&'[') {
                            chars.next(); // consume second [
                            bracket_count += 1;
                        }
                    }
                } else {
                    result.push(ch);
                }
            }
            '-' => {
                // Check for comment
                if chars.peek() == Some(&'-') {
                    chars.next(); // consume second -
                    // Skip rest of line
                    while let Some(inner_ch) = chars.next() {
                        if inner_ch == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                } else {
                    result.push(ch);
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

fn is_balanced(input: &str) -> bool {
    let mut paren_count = 0;
    let mut brace_count = 0;
    let mut bracket_count = 0;

    for ch in input.chars() {
        match ch {
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }

        // If any count goes negative, we have a closing bracket without an opening one
        if paren_count < 0 || brace_count < 0 || bracket_count < 0 {
            return false;
        }
    }

    paren_count == 0 && brace_count == 0 && bracket_count == 0
}

fn has_incomplete_control_structure(input: &str) -> bool {
    let input = input.to_lowercase();
    let lines: Vec<&str> = input.lines().collect();

    // Count control structure keywords
    let mut if_count = 0;
    let mut for_count = 0;
    let mut while_count = 0;
    let mut repeat_count = 0;
    let mut function_count = 0;

    let mut end_count = 0;
    let mut until_count = 0;

    for line in lines {
        let words: Vec<&str> = line.split_whitespace().collect();
        for word in words {
            match word {
                "if" => if_count += 1,
                "elseif" => {}, // elseif is part of the if statement, doesn't need its own end
                "for" => for_count += 1,
                "while" => while_count += 1,
                "repeat" => repeat_count += 1,
                "function" => function_count += 1,
                "end" => end_count += 1,
                "until" => until_count += 1,
                _ => {}
            }
        }
    }

    // Check if control structures are balanced
    let expected_ends = if_count + for_count + while_count + repeat_count + function_count;
    let actual_ends = end_count + until_count;

    expected_ends > actual_ends
}

fn is_incomplete_syntax_error(error: &str) -> bool {
    let error_lower = error.to_lowercase();

    // Common patterns that indicate incomplete input
    error_lower.contains("unexpected end of file") ||
        error_lower.contains("'end' expected") ||
        error_lower.contains("'until' expected") ||
        error_lower.contains("unexpected symbol near <eof>") ||
        error_lower.contains("unfinished string") ||
        error_lower.contains("missing closing") ||
        error_lower.contains("incomplete")
}

fn clean_lua_error(err: &str) -> String {
    let start_str = "src/repl.rs:";
    if let Some(pos) =  err.find(start_str) {
        let err = &err[pos + start_str.len()..];
        if let Some(pos2) =  err.find(':') {
            let err = &err[pos2 + 1..];
            if let Some(pos2) =  err.find(':') {
                let err = &err[pos2 + 1..];
                return err.trim().to_string();
            }
        }
    }
    err.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_balanced_parentheses() {
        // Balanced cases
        assert!(is_balanced("()"));
        assert!(is_balanced("(()())"));
        assert!(is_balanced("print(hello())"));
        assert!(is_balanced(""));
        assert!(is_balanced("no brackets here"));

        // Unbalanced cases
        assert!(!is_balanced("("));
        assert!(!is_balanced(")"));
        assert!(!is_balanced("(()"));
        assert!(!is_balanced("())"));
        assert!(!is_balanced("((())"));
    }

    #[test]
    fn test_is_balanced_braces() {
        // Balanced cases
        assert!(is_balanced("{}"));
        assert!(is_balanced("{a = 1, b = 2}"));
        assert!(is_balanced("{{}, {}}"));

        // Unbalanced cases
        assert!(!is_balanced("{"));
        assert!(!is_balanced("}"));
        assert!(!is_balanced("{{{}}"));
        assert!(!is_balanced("{{}"));
    }

    #[test]
    fn test_is_balanced_brackets() {
        // Balanced cases
        assert!(is_balanced("[]"));
        assert!(is_balanced("[1, 2, 3]"));
        assert!(is_balanced("[[]]"));

        // Unbalanced cases
        assert!(!is_balanced("["));
        assert!(!is_balanced("]"));
        assert!(!is_balanced("[[]"));
        assert!(!is_balanced("[]]"));
    }

    #[test]
    fn test_is_balanced_mixed() {
        // Balanced mixed cases
        assert!(is_balanced("({[]})"));
        assert!(is_balanced("table[key](value)"));
        assert!(is_balanced("func({a = [1, 2]})"));

        // Unbalanced mixed case
        assert!(!is_balanced("({[)]})"));
    }

    #[test]
    fn test_remove_strings_and_comments() {
        // Double quoted strings
        assert_eq!(remove_strings_and_comments("print(\"hello world\")"), "print()");
        assert_eq!(remove_strings_and_comments("\"({[\""), "");

        // Single quoted strings
        assert_eq!(remove_strings_and_comments("print('hello world')"), "print()");
        assert_eq!(remove_strings_and_comments("'({['"), "");

        // Escaped quotes
        assert_eq!(remove_strings_and_comments("\"hello \\\"world\\\"\""), "");
        assert_eq!(remove_strings_and_comments("'hello \\'world\\''"), "");

        // Comments
        assert_eq!(remove_strings_and_comments("code -- this is a comment"), "code ");
        assert_eq!(remove_strings_and_comments("-- full line comment\ncode"), "\ncode");

        // Long strings
        assert_eq!(remove_strings_and_comments("[[long string with ({[]})]]"), "");
        assert_eq!(remove_strings_and_comments("code [[string]] more"), "code  more");

        // Mixed cases
        assert_eq!(
            remove_strings_and_comments("print(\"hello\") -- comment with ({["),
            "print() "
        );
    }

    #[test]
    fn test_has_incomplete_control_structure() {
        // Complete control structures
        assert!(!has_incomplete_control_structure("if true then print('hi') end"));
        assert!(!has_incomplete_control_structure("for i = 1, 10 do print(i) end"));
        assert!(!has_incomplete_control_structure("while true do break end"));
        assert!(!has_incomplete_control_structure("function test() return 1 end"));
        assert!(!has_incomplete_control_structure("repeat print('hi') until false"));

        // Incomplete control structures
        assert!(has_incomplete_control_structure("if true then"));
        assert!(has_incomplete_control_structure("for i = 1, 10 do"));
        assert!(has_incomplete_control_structure("while true do"));
        assert!(has_incomplete_control_structure("function test()"));
        assert!(has_incomplete_control_structure("repeat"));

        // Nested complete structures
        assert!(!has_incomplete_control_structure(
            "if true then\n  for i = 1, 10 do\n    print(i)\n  end\nend"
        ));

        // Nested incomplete structures
        assert!(has_incomplete_control_structure(
            "if true then\n  for i = 1, 10 do\n    print(i)\n  end"
        ));

        // Multiple incomplete structures
        assert!(has_incomplete_control_structure("if true then\nfor i = 1, 10 do"));

        // elseif handling
        assert!(has_incomplete_control_structure("if true then\nprint('hi')\nelseif false then"));
        assert!(!has_incomplete_control_structure("if true then\nprint('hi')\nelseif false then\nprint('bye')\nend"));
    }

    #[test]
    fn test_is_incomplete_syntax_error() {
        // Incomplete syntax error messages
        assert!(is_incomplete_syntax_error("unexpected end of file"));
        assert!(is_incomplete_syntax_error("'end' expected"));
        assert!(is_incomplete_syntax_error("'until' expected"));
        assert!(is_incomplete_syntax_error("unexpected symbol near <eof>"));
        assert!(is_incomplete_syntax_error("unfinished string"));
        assert!(is_incomplete_syntax_error("missing closing bracket"));
        assert!(is_incomplete_syntax_error("incomplete statement"));

        // Complete syntax error messages (not incomplete)
        assert!(!is_incomplete_syntax_error("attempt to call a nil value"));
        assert!(!is_incomplete_syntax_error("undefined variable"));
        assert!(!is_incomplete_syntax_error("syntax error"));
        assert!(!is_incomplete_syntax_error("invalid escape sequence"));

        // Case insensitive
        assert!(is_incomplete_syntax_error("UNEXPECTED END OF FILE"));
        assert!(is_incomplete_syntax_error("'END' EXPECTED"));
    }

    #[test]
    fn test_is_complete_statement_simple() {
        // Simple complete statements
        assert!(is_complete_statement("print('hello')"));
        assert!(is_complete_statement("local x = 5"));
        assert!(is_complete_statement("return 42"));
        assert!(is_complete_statement("x = x + 1"));

        // Simple incomplete statements (unbalanced)
        assert!(!is_complete_statement("print('hello'"));
        assert!(!is_complete_statement("local x = {"));
        assert!(!is_complete_statement("func("));
    }

    #[test]
    fn test_is_complete_statement_control_structures() {
        // Complete control structures
        assert!(is_complete_statement("if true then print('hi') end"));
        assert!(is_complete_statement("for i = 1, 10 do print(i) end"));
        assert!(is_complete_statement("while true do break end"));
        assert!(is_complete_statement("function test() return 1 end"));
        assert!(is_complete_statement("repeat print('hi') until false"));

        // Incomplete control structures
        assert!(!is_complete_statement("if true then"));
        assert!(!is_complete_statement("for i = 1, 10 do"));
        assert!(!is_complete_statement("while true do"));
        assert!(!is_complete_statement("function test()"));
        assert!(!is_complete_statement("repeat"));
    }

    #[test]
    fn test_is_complete_statement_multiline() {
        // Complete multiline statements
        assert!(is_complete_statement("local t = {\n  x = 1,\n  y = 2\n}"));
        assert!(is_complete_statement("function greet(name)\n  print('Hello, ' .. name)\nend"));
        assert!(is_complete_statement("if x > 0 then\n  print('positive')\nelse\n  print('not positive')\nend"));

        // Incomplete multiline statements
        assert!(!is_complete_statement("local t = {\n  x = 1,\n  y = 2"));
        assert!(!is_complete_statement("function greet(name)\n  print('Hello, ' .. name)"));
        assert!(!is_complete_statement("if x > 0 then\n  print('positive')\nelse\n  print('not positive')"));
    }

    #[test]
    fn test_is_complete_statement_with_strings_and_comments() {
        // Complete statements with strings containing braces
        assert!(is_complete_statement("print('hello {world}')"));
        assert!(is_complete_statement("local s = \"string with } brace\""));

        // Complete statements with comments containing braces
        assert!(is_complete_statement("print('hello') -- comment with {"));
        assert!(is_complete_statement("local x = 5 -- comment with }"));

        // Incomplete statements should still be detected despite strings/comments
        assert!(!is_complete_statement("if true then -- comment with }"));
        assert!(!is_complete_statement("local t = { -- comment\n  x = 1"));
    }

    #[test]
    fn test_is_complete_statement_edge_cases() {
        // Empty string
        assert!(is_complete_statement(""));

        // Only whitespace
        assert!(is_complete_statement("   \n  \t  "));

        // Only comments
        assert!(is_complete_statement("-- just a comment"));
        assert!(is_complete_statement("-- comment 1\n-- comment 2"));

        // Mixed statements
        assert!(is_complete_statement("local x = 5; print(x)"));
        assert!(!is_complete_statement("local x = 5; if x > 0 then"));
    }

    #[test]
    fn test_clean_lua_error() {
        let cleaned = clean_lua_error("src/repl.rs:70:1: syntax error near 'das'");
        assert_eq!(cleaned, "syntax error near 'das'");
        let cleaned = clean_lua_error("syntax error: src/repl.rs:70:1: syntax error near '-'");
        assert_eq!(cleaned, "syntax error near '-'");
    }

    #[test]
    fn test_integration_multiline_scenarios() {
        // Table definition
        let table_def = "local t = {\n  name = \"test\",\n  value = 42\n}";
        assert!(is_complete_statement(table_def));

        // Function definition
        let func_def = "function calculate(a, b)\n  return a + b\nend";
        assert!(is_complete_statement(func_def));

        // Nested control structures
        let nested = "for i = 1, 10 do\n  if i % 2 == 0 then\n    print(i)\n  end\nend";
        assert!(is_complete_statement(nested));

        // Incomplete nested structures
        let incomplete_nested = "for i = 1, 10 do\n  if i % 2 == 0 then\n    print(i)\n  end";
        assert!(!is_complete_statement(incomplete_nested));
    }

    #[test]
    fn test_lua_specific_constructs() {
        // Local function
        assert!(is_complete_statement("local function test() return 1 end"));
        assert!(!is_complete_statement("local function test()"));

        // Anonymous function
        assert!(is_complete_statement("local f = function() return 1 end"));
        assert!(!is_complete_statement("local f = function()"));

        // Do blocks
        assert!(is_complete_statement("do local x = 1; print(x) end"));
        assert!(!is_complete_statement("do local x = 1"));

        // Multiple assignment
        assert!(is_complete_statement("local a, b = 1, 2"));
        assert!(!is_complete_statement("local a, b = func("));

        // Repeat-until
        assert!(is_complete_statement("repeat x = x + 1 until x > 10"));
        assert!(!is_complete_statement("repeat x = x + 1"));
    }
}
