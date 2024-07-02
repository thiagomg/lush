mod compression;
mod environment;
mod cmd_line;

use std::{env, fs};
use std::path::PathBuf;
use clap::Parser;
use mlua::prelude::*;
use mlua::{Value, Variadic};
use crate::cmd_line::Args;
use crate::compression::zip_deflate;

fn set_utils(lua: &Lua) -> LuaResult<()> {

    // Compression
    // TODO: How to accept varargs? table???
    let zip_f = lua.create_function(|_, (zip_name, files_to_add): (String, Variadic<Value>), | {
        let mut files = vec![];
        for (i, arg) in files_to_add.iter().enumerate() {
            println!("{} = {:?}", i, arg.to_string()?);
            files.push(PathBuf::from(arg.to_string()?));
        }
        println!("zip={}, files={:?}", zip_name, files);
        zip_deflate(&PathBuf::from(&zip_name), &files, true)?;
        Ok(())
    })?;

    let compression_tb = lua.create_table()?;
    compression_tb.set("zip", zip_f)?;
    lua.globals().set("compression", compression_tb)?;

    // Env
    let chdir_f = lua.create_function(|_, new_dir: String, | {
        env::set_current_dir(new_dir)?;
        Ok(())
    })?;

    // TODO: Remove this function
    let print_f = lua.create_function(|_, args: Variadic<Value>, | {
        for (i, arg) in args.iter().enumerate() {
            println!("{} = {:?}", i, arg.to_string()?);
        }
        Ok(())
    })?;

    let env_tb = lua.create_table()?;
    env_tb.set("cd", chdir_f)?;
    env_tb.set("print", print_f)?;
    lua.globals().set("env", env_tb)?;

    Ok(())
}

fn run_file(script: &String) -> LuaResult<()> {
    let lua = Lua::new();
    set_utils(&lua)?;

    lua.load(script).exec()?;

    Ok(())
}

// TODO: chdir, mkdir, cp, mv, ls, pushd, popd, getenv, setenv
// println!("{}", env::consts::OS);

fn main() -> LuaResult<()> {
    //let input_file = PathBuf::from("/Users/thiago/src/lush/scripts/test.lua");

    let args = Args::parse();
    let input_file = args.lua_file;
    let script = fs::read_to_string(input_file).expect("Error opening input file");
    // TODO: Parse result
    run_file(&script)
}
