use mlua::{Lua, Value, Result};
use rustyline::{ColorMode, Config, DefaultEditor, Editor};
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
    rl.set_helper(Some(LushHighlighter::default()));

    println!("{}. Press {} or type `{}` to quit.", "LuSH REPL".bold(), "Ctrl+D".bold(), "exit".bold());

    loop {
        // let prompt = "lush> ".green().to_string();
        let prompt = "lush> ";
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed == "exit" {
                    break;
                }

                rl.add_history_entry(trimmed).ok();

                match lua.load(trimmed).eval::<Value>() {
                    Ok(Value::Nil) => { println!("Nil") } // don't print anything
                    Ok(result) => println!("=> {:?}", result),
                    Err(err) => {
                        eprintln!("Error: {}", clean_lua_error(&err.to_string()).red());
                    }
                }
            }
            Err(_) => break, // e.g., Ctrl+D
        }
    }

    Ok(())
}

fn clean_lua_error(err: &str) -> String {
    err.lines()
        .filter(|line| line.contains("[string") || line.contains(".lua") || !line.contains("src/"))
        .collect::<Vec<_>>()
        .join("\n")
}

