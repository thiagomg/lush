mod compression;
mod environment;
mod cmd_line;
mod filesystem;
mod os;

use std::fs;
use std::path::PathBuf;
use clap::Parser;
use mlua::prelude::*;
use crate::cmd_line::Args;
use crate::compression::zip_deflate;
use crate::environment::{chdir, popd, pushd, pwd};
use crate::filesystem::{ls, mkdir, rmdir};
use crate::os::os_name;

struct LushContext {
    dir_stack: Vec<PathBuf>,
}

fn set_utils(lua: &Lua) -> LuaResult<()> {
    // Env
    let pushd_f = lua.create_function_mut(pushd)?;
    let popd_f = lua.create_function_mut(popd)?;
    let chdir_f = lua.create_function(chdir)?;
    let pwd_f = lua.create_function(pwd)?;

    let env_tb = lua.create_table()?;
    env_tb.set("cd", chdir_f)?;
    env_tb.set("pushd", pushd_f)?;
    env_tb.set("popd", popd_f)?;
    env_tb.set("pwd", pwd_f)?;
    lua.globals().set("env", env_tb)?;

    // File System
    let ls_f = lua.create_function(ls)?;
    let mkdir_f = lua.create_function(mkdir)?;
    let rmdir_f = lua.create_function(rmdir)?;
    let filesystem_tb = lua.create_table()?;
    filesystem_tb.set("ls", ls_f)?;
    filesystem_tb.set("mkdir", mkdir_f)?;
    filesystem_tb.set("rmdir", rmdir_f)?;
    lua.globals().set("fs", filesystem_tb)?;

    // Operating System
    let os_name_f = lua.create_function(os_name)?;
    let os_tb = lua.create_table()?;
    os_tb.set("name", os_name_f)?;
    lua.globals().set("os", os_tb)?;

    // Compression
    let zip_f = lua.create_function(zip_deflate)?;
    let compression_tb = lua.create_table()?;
    compression_tb.set("zip", zip_f)?;
    lua.globals().set("compression", compression_tb)?;

    Ok(())
}

fn run_file(script: &String) -> LuaResult<()> {
    let lua = Lua::new();
    let ctx = LushContext {
        dir_stack: vec![],
    };
    lua.set_app_data(ctx);
    set_utils(&lua)?;

    lua.load(script).exec()?;

    Ok(())
}


// TODO: mkdir, cp, mv, getenv, setenv
// println!("{}", env::consts::OS);

fn main() -> LuaResult<()> {
    // let input_file = PathBuf::from("/Users/thiago/src/lush/scripts/test.lua");

    let args = Args::parse();
    let input_file = args.lua_file;
    let script = fs::read_to_string(input_file).expect("Error opening input file");
    run_file(&script)
}
