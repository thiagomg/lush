use std::path::PathBuf;
use mlua::Lua;
use mlua::prelude::LuaResult;
use crate::files::{zip_deflate, zip_inflate};
use crate::environment::{chdir, get_env, popd, print, pushd, pwd, rem_env, set_env};
use crate::filesystem::{copy_file, file_exists, ls, mkdir, move_file, rmdir};
use crate::os::{os_name, proc_exes, proc_names};

pub(crate) struct LushContext {
    pub dir_stack: Vec<PathBuf>,
}

// TODO: Creation of temporary dir (e.g. mktemp)

fn set_utils(lua: &Lua) -> LuaResult<()> {
    // Env
    let env_tb = lua.create_table()?;
    env_tb.set("cd", lua.create_function(chdir)?)?;
    env_tb.set("pushd", lua.create_function_mut(pushd)?)?;
    env_tb.set("popd", lua.create_function_mut(popd)?)?;
    env_tb.set("pwd", lua.create_function(pwd)?)?;
    env_tb.set("get", lua.create_function(get_env)?)?;
    env_tb.set("set", lua.create_function(set_env)?)?;
    env_tb.set("del", lua.create_function(rem_env)?)?;
    env_tb.set("print", lua.create_function(print)?)?;
    lua.globals().set("env", env_tb)?;

    // File System
    let filesystem_tb = lua.create_table()?;
    filesystem_tb.set("ls", lua.create_function(ls)?)?;
    filesystem_tb.set("mkdir", lua.create_function(mkdir)?)?;
    filesystem_tb.set("rmdir", lua.create_function(rmdir)?)?;
    filesystem_tb.set("copy", lua.create_function(copy_file)?)?;
    filesystem_tb.set("move", lua.create_function(move_file)?)?;
    filesystem_tb.set("exists", lua.create_function(file_exists)?)?;
    lua.globals().set("fs", filesystem_tb)?;

    // Operating System
    let os_tb: mlua::Table = lua.globals().get("os")?;
    os_tb.set("name", lua.create_function(os_name)?)?;
    os_tb.set("proc_names", lua.create_function(proc_names)?)?;
    os_tb.set("proc_exes", lua.create_function(proc_exes)?)?;
    
    // Compression
    let files_tb = lua.create_table()?;
    files_tb.set("zip", lua.create_function(zip_deflate)?)?;
    files_tb.set("unzip", lua.create_function(zip_inflate)?)?;
    lua.globals().set("files", files_tb)?;

    Ok(())
}

pub(crate) fn run_script(script: &str) -> LuaResult<()> {
    let lua = Lua::new();
    let ctx = LushContext {
        dir_stack: vec![],
    };
    lua.set_app_data(ctx);
    set_utils(&lua)?;

    lua.load(script).exec()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test::data::DATA;
    use super::*;

    #[test]
    fn run_test_script() {
        run_script(DATA).unwrap();
    }

    #[test]
    fn run_builtin_os() {
        run_script("os.execute('ls')").unwrap();
    }

    #[test]
    fn invalid_function() {
        let data = r##"

        env.invalid_func(1)
        "##;
        let res = run_script(data).unwrap_err();

        let ts = res.to_string();
        println!("{}", ts);
    }
}