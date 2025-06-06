use mlua::{Lua, Value, Result};
use rustyline::DefaultEditor;
use crate::setup;
use crate::setup::LushContext;

use colored::Colorize;

pub fn run_repl() -> Result<()> {
    let lua = Lua::new();
    let ctx = LushContext {
        dir_stack: vec![],
    };
    lua.set_app_data(ctx);
    setup::set_utils(&lua)?;
    let mut rl = DefaultEditor::new().expect("Could not create line editor");

    println!("LuSH REPL. Press Ctrl+D or type `exit` to quit.");

    loop {
        let readline = rl.readline("lush> ");
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

