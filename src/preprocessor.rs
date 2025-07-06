use regex::Regex;

pub fn interpolate_strings(text: &str) -> String {
    let re_string = Regex::new(r#""((?:[^"\\]|\\.)*)""#).unwrap();
    let re_interpolation = Regex::new(r#"\$\{(.*?)}"#).unwrap();

    let res = re_string.replace_all(text, |str_cap: &regex::Captures| {
        let string_val = &str_cap[1];
        let new_string = re_interpolation.replace_all(string_val, |var_cap: &regex::Captures| {
            let var_name = &var_cap[1];
            format!("\" .. tostring({}) .. \"", var_name)
        });
        format!(r#""{}""#, new_string)
    });

    res.to_string()
}

fn parse_shell_command(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut chars = cmd.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                // Double quotes are not added in the argument
            }
            ' ' | '\t' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
                // Pula espaços em branco consecutivos
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ' ' || next_ch == '\t' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

fn get_lua_commands(commands: Vec<&str>) -> Vec<String> {
    let lua_commands: Vec<String> = commands.iter().map(|cmd| {
        if cmd.starts_with("`") && cmd.ends_with("`") {
            // If it's a lua function, it needs to be surrounded by "`"
            let lua_func = &cmd[1..cmd.len() - 1];
            format!("{{{}}}", lua_func)
        } else {
            // Otherwise, we consider it a shell command
            let parts = parse_shell_command(cmd);
            let formatted_parts: Vec<String> = parts.iter().map(|part| {
                format!("\"{}\"", part)
            }).collect();
            format!("{{{}}}", formatted_parts.join(", "))

        }
    }).collect();
    lua_commands
}

fn is_inside_single_quotes(text: &str, position: usize) -> bool {
    let mut in_single_quotes = false;
    let mut escaped = false;
    
    for (i, ch) in text.char_indices() {
        if i >= position {
            break;
        }
        
        if escaped {
            escaped = false;
            continue;
        }
        
        match ch {
            '\\' => escaped = true,
            '\'' => in_single_quotes = !in_single_quotes,
            _ => {}
        }
    }
    
    in_single_quotes
}

fn process_captures(text: &str, re: Regex, fmt_func: fn(String) -> String) -> String {
    let result = re.replace_all(text, |captures: &regex::Captures| {
        let match_start = captures.get(0).unwrap().start();

        // Check if this match is inside single quotes
        if is_inside_single_quotes(text, match_start) {
            return captures[0].to_string(); // Return the original match unchanged
        }

        let shell_command = captures[1].trim();
        let commands: Vec<&str> = shell_command.split('|').map(|s| s.trim()).collect();
        let lua_commands = get_lua_commands(commands);
        // format!("os.pipe_exec({})", lua_commands.join(", "))
        fmt_func(lua_commands.join(", "))
    });
    result.to_string()
}

pub fn replace_shell_exec(text: &str) -> String {
    let re = Regex::new(r"\$>\s*([^\r\n]+)").unwrap();
    process_captures(text, re, |arguments: String| format!("os.pipe_exec({})", arguments))
}

pub fn replace_sub_shell(text: &str) -> String {
    let re = Regex::new(r"\$\(([^)]+)\)").unwrap();
    process_captures(text, re, |arguments: String| format!("os.pipeline({})", arguments))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_strings_test() {
        let source = r#"
local a = "this is not going to be replaced"
local b = "this is going to be ${found} because it has the special pattern"
local c = "this \"might\" fail and it has ${another} too"
"#;

        let expected = r#"
local a = "this is not going to be replaced"
local b = "this is going to be " .. tostring(found) .. " because it has the special pattern"
local c = "this \"might\" fail and it has " .. tostring(another) .. " too"
"#;

        let res = interpolate_strings(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn interpolate_strings_start_end() {
        let source = r#""${var} in the start""#;
        let res = interpolate_strings(source);
        assert_eq!(res, r#""" .. tostring(var) .. " in the start""#);

        let source = r#""now in the ${end}""#;
        let res = interpolate_strings(source);
        assert_eq!(res, r#""now in the " .. tostring(end) .. """#);
    }

    #[test]
    fn replace_shell_exec_test() {
        let source = r#"
$> ls -la | grep error
print("hello")
$> cat file.txt | head -10 | tail -5
"#;

        let expected = r#"
os.pipe_exec({"ls", "-la"}, {"grep", "error"})
print("hello")
os.pipe_exec({"cat", "file.txt"}, {"head", "-10"}, {"tail", "-5"})
"#;

        let res = replace_shell_exec(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn replace_shell_exec_single_command() {
        let source = r#"$> ls -la"#;
        let res = replace_shell_exec(source);
        assert_eq!(res, r#"os.pipe_exec({"ls", "-la"})"#);
    }

    #[test]
    fn replace_shell_exec_no_match() {
        let source = r#"print("no shell commands here")"#;
        let res = replace_shell_exec(source);
        assert_eq!(res, source);
    }

    #[test]
    fn replace_sub_shell_test() {
        let source = r#"
local res = $(ls -la | grep error)
print("hello")
local output = $(cat file.txt | head -10 | tail -5)
local simple = $(pwd)
"#;

        let expected = r#"
local res = os.pipeline({"ls", "-la"}, {"grep", "error"})
print("hello")
local output = os.pipeline({"cat", "file.txt"}, {"head", "-10"}, {"tail", "-5"})
local simple = os.pipeline({"pwd"})
"#;

        let res = replace_sub_shell(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn replace_sub_shell_single_command() {
        let source = r#"local date = $(date)"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, r#"local date = os.pipeline({"date"})"#);
    }

    #[test]
    fn replace_sub_shell_single_command_with_spaces() {
        let source = r#"local date = $(  date  )"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, r#"local date = os.pipeline({"date"})"#);
    }

    #[test]
    fn replace_sub_shell_multiple_in_line() {
        let source = r#"local combined = $(ls) .. " and " .. $(pwd)"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, r#"local combined = os.pipeline({"ls"}) .. " and " .. os.pipeline({"pwd"})"#);
    }

    #[test]
    fn replace_sub_shell_no_match() {
        let source = r#"print("no sub shell here")"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, source);
    }

    #[test]
    fn replace_sub_shell_with_args() {
        let source = r#"local files = $(find . -name "*.lua")"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, r#"local files = os.pipeline({"find", ".", "-name", "*.lua"})"#);
    }

    #[test]
    fn replace_shell_exec_with_quoted_args() {
        let source = r#"$> tail /tmp/my-file.log | grep "error 2""#;
        let expected = r#"os.pipe_exec({"tail", "/tmp/my-file.log"}, {"grep", "error 2"})"#;

        let res = replace_shell_exec(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn replace_shell_exec_with_quoted_args_and_lua_func() {
        let source = r#"$> tail /tmp/my-file.log | `in_brackets` | grep "error 2""#;
        let expected = r#"os.pipe_exec({"tail", "/tmp/my-file.log"}, {in_brackets}, {"grep", "error 2"})"#;

        let res = replace_shell_exec(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn replace_sub_shell_with_quoted_args_and_lua_func() {
        let source = r#"local res = $(tail /tmp/my-file.log | `in_brackets` | grep "error 2")"#;
        let expected = r#"local res = os.pipeline({"tail", "/tmp/my-file.log"}, {in_brackets}, {"grep", "error 2"})"#;

        let res = replace_sub_shell(source);
        assert_eq!(res, expected);
    }
    
    #[test]
    fn replace_shell_exec_with_quoted_args_no_space() {
        let source = r#"$>tail /tmp/my-file.log | grep "error 2""#;
        let expected = r#"os.pipe_exec({"tail", "/tmp/my-file.log"}, {"grep", "error 2"})"#;

        let res = replace_shell_exec(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn replace_sub_shell_with_quoted_args_fixed() {
        let source = r#"local files = $(find . -name "*.lua")"#;
        let expected = r#"local files = os.pipeline({"find", ".", "-name", "*.lua"})"#;

        let res = replace_sub_shell(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn replace_sub_shell_inside_single_quotes() {
        let source = r#"local x = '$(ls)'"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, r#"local x = '$(ls)'"#);
    }

    #[test]
    fn replace_sub_shell_inside_single_quotes_and_spaces() {
        let source = r#"local x = 'this is a command $(ls) that is not replaced'"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, r#"local x = 'this is a command $(ls) that is not replaced'"#);
    }


    #[test]
    fn replace_shell_exec_inside_single_quotes() {
        let source = r#"local x = '$> ls -la'"#;
        let res = replace_shell_exec(source);
        assert_eq!(res, r#"local x = '$> ls -la'"#);
    }

    #[test]
    fn replace_sub_shell_mixed_quotes() {
        let source = r#"local x = '$(ls)' .. $(pwd)"#;
        let expected = r#"local x = '$(ls)' .. os.pipeline({"pwd"})"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, expected);
    }

    #[test]
    fn replace_sub_shell_escaped_single_quotes() {
        let source = r#"local x = 'Don\'t replace $(ls) here'"#;
        let res = replace_sub_shell(source);
        assert_eq!(res, r#"local x = 'Don\'t replace $(ls) here'"#);
    }
}