mod files;
mod environment;
mod cmd_line;
mod filesystem;
mod os;
mod setup;
mod test;

use std::fs;
use clap::Parser;
use colored::Colorize;
use regex::Regex;
use crate::cmd_line::Args;
use crate::setup::run_file;

fn main() -> Result<(), String> {
    let args = Args::parse();
    let input_file = args.lua_file;
    let script = fs::read_to_string(input_file.clone()).expect("Error opening input file");
    let res = run_file(&script);
    if let Err(e) = res {
        println!();
        let error_desc = e.to_string();

        let err_desc = if let Some(line) = line_number_from_err(&error_desc) {
            format!("Error parsing script {}, line {}", input_file.to_str().unwrap().to_string(), line)
        } else {
            format!("Error parsing script {}", input_file.to_str().unwrap().to_string())
        };

        println!("{}", err_desc.red());
        println!("{}", e);
        
        return Err(err_desc);
    }

    Ok(())
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
        let script = fs::read_to_string("scripts/test.lua").expect("Error opening input file");
        let res = run_file(&script);
        assert!(res.is_ok());
    }
}