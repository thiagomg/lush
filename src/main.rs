mod files;
mod environment;
mod cmd_line;
mod filesystem;
mod os;
mod setup;

use std::fs;
use clap::Parser;
use mlua::prelude::*;
use crate::cmd_line::Args;
use crate::setup::run_file;

fn main() -> LuaResult<()> {
    let args = Args::parse();
    let input_file = args.lua_file;
    let script = fs::read_to_string(input_file).expect("Error opening input file");
    run_file(&script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_test_file() {
        let script = fs::read_to_string("scripts/test.lua").expect("Error opening input file");
        run_file(&script).unwrap();
    }
}