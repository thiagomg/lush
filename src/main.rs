mod files;
mod environment;
mod cmd_line;
mod filesystem;
mod os;
mod setup;
mod test;
mod string_utils;
mod pipeline_exec;

use std::fs;
use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;
use mlua::Error;
use regex::Regex;
use crate::cmd_line::Args;
use crate::setup::run_script;
use crate::string_utils::remove_shebang;


fn main() -> Result<(), String> {
    let args = Args::parse();
    let input_file = args.lua_file;
    run_file(input_file)
}

fn run_file(input_file: PathBuf) -> Result<(), String> {
    let script = fs::read_to_string(input_file.clone()).expect("Error opening input file");
    let script = remove_shebang(script);
    let res = run_script(&script);
    if let Err(e) = res {
        let error_desc = e.to_string();
        let err_prefix = format!("Error parsing script {}", input_file.to_str().unwrap());
        let err_desc = if let Some(line) = line_number_from_err(&error_desc) {
            format!("{}, line {}", err_prefix, line)
        } else {
            err_prefix
        };

        println!("{}", err_desc.red().bold());

        print_error(&e);
        return Err("Error parsing lush file".to_string());
    }

    Ok(())
}

fn print_error(error: &Error) {
    match error {
        Error::SyntaxError { message, .. } => println!("{}: {}", "Syntax error".bold(), message),
        Error::RuntimeError(msg) => println!("{}: {}", "Runtime error".bold(), msg),
        Error::CallbackError { traceback: _traceback, cause } => {
            print_error(cause);
        }
        _ => {
            println!("{}", error);
        }
    }
}

fn line_number_from_err(error_message: &str) -> Option<usize> {
    let re = Regex::new(r"\[string.+?]:(\d+)").unwrap();
    if let Some(caps) = re.captures(error_message) {
        if let Some(matched) = caps.get(1) {
            return matched.as_str().parse::<usize>().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_test_file() {
        let res = run_file(PathBuf::from("scripts/test.lua"));
        assert!(res.is_ok());
    }

    #[test]
    fn test_run_pipeline() {
        let res = run_file(PathBuf::from("scripts/pipetest.lua"));
        assert!(res.is_ok());
    }
}