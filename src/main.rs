mod cmd_line;
mod setup;
mod test;
mod string_utils;

mod compress;
mod utils;
mod repl;
mod modules;
mod lush_highlighter;
mod preprocessor;

use std::{env, fs};
use std::path::PathBuf;
use std::sync::Mutex;
use clap::Parser;
use colored::Colorize;
use crate::cmd_line::Args;
use crate::repl::run_repl;
use crate::setup::run_script;
use crate::string_utils::remove_shebang;

use once_cell::sync::Lazy;

pub static TEMP_PATHS: Lazy<Mutex<Vec<PathBuf>>> = Lazy::new(|| Mutex::new(vec![]));

fn main() {
    let cmd_line_args = env::args().skip(1).collect::<Vec<_>>();
    if cmd_line_args.is_empty() {
        if let Err(e) = run_repl() {
            eprintln!("{}", e.to_string().red());
            std::process::exit(2);
        }
        std::process::exit(0);
    }

    let args = Args::parse();
    let input_file = args.lua_file;
    match run_file(input_file, args.script_args) {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            println!("{}", error.to_string().red());
            std::process::exit(1);
        }
    }
}

fn run_file(input_file: PathBuf, args: Vec<String>) -> Result<(), String> {
    let script = fs::read_to_string(input_file.clone())
        .unwrap_or_else(|_| panic!("Error opening input file {}", input_file.display()));
    let script = remove_shebang(script);
    let res = run_script(&script, input_file.clone(), args).map_err(|error| error.to_string());

    let paths = TEMP_PATHS.lock().unwrap();
    for path in paths.iter() {
        if path.is_file() {
            let _ = fs::remove_file(path);
        } else if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        }
    }
    
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_test_file() {
        let res = run_file(PathBuf::from("scripts/test.lua"), vec![]);
        println!("{:?}", res);
        match res {
            Ok(()) => println!("ok"),
            Err(error) => {
                println!("{}", error.to_string().red());
            }
        }
    }

    #[test]
    fn test_run_pipeline() {
        let res = run_file(PathBuf::from("scripts/pipetest.lua"), vec![]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_run_2() {
        let res = run_file(PathBuf::from("scripts/test2.lush"), vec![]);
        assert!(res.is_ok());
    }
}