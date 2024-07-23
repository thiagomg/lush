use std::path::PathBuf;
use mlua::Lua;
use mlua::prelude::LuaResult;
use crate::files::{zip_deflate, zip_inflate};
use crate::environment::{chdir, get_env, popd, print, pushd, pwd, rem_env, set_env};
use crate::filesystem::{ls, mkdir, rmdir};
use crate::os::os_name;

pub(crate) struct LushContext {
    pub dir_stack: Vec<PathBuf>,
}

// TODO: Creation of temporary dir (e.g. mktemp)

fn set_utils(lua: &Lua) -> LuaResult<()> {
    // Env
    let pushd_f = lua.create_function_mut(pushd)?;
    let popd_f = lua.create_function_mut(popd)?;
    let chdir_f = lua.create_function(chdir)?;
    let pwd_f = lua.create_function(pwd)?;
    let get_env_f = lua.create_function(get_env)?;
    let set_env_f = lua.create_function(set_env)?;
    let rem_env_f = lua.create_function(rem_env)?;
    let print_f = lua.create_function(print)?;

    let env_tb = lua.create_table()?;
    env_tb.set("cd", chdir_f)?;
    env_tb.set("pushd", pushd_f)?;
    env_tb.set("popd", popd_f)?;
    env_tb.set("pwd", pwd_f)?;
    env_tb.set("get", get_env_f)?;
    env_tb.set("set", set_env_f)?;
    env_tb.set("del", rem_env_f)?;
    env_tb.set("print", print_f)?;
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

    let os_tb: mlua::Table = lua.globals().get("os")?;
    os_tb.set("name", os_name_f)?;


    // Compression
    let zip_f = lua.create_function(zip_deflate)?;
    let unzip_f = lua.create_function(zip_inflate)?;
    let files_tb = lua.create_table()?;
    files_tb.set("zip", zip_f)?;
    files_tb.set("unzip", unzip_f)?;
    lua.globals().set("files", files_tb)?;

    Ok(())
}

pub(crate) fn run_file(script: &str) -> LuaResult<()> {
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
        run_file(DATA).unwrap();
    }

    #[test]
    fn run_builtin_os() {
        run_file("os.execute('ls')").unwrap();
    }

    #[test]
    fn invalid_function() {
        let data = r##"

        env.invalid_func(1)
        "##;
        let res = run_file(data).unwrap_err();

        let ts = res.to_string();
        println!("{}", ts);
    }
}