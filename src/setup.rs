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
    let os_tb = lua.create_table()?;
    os_tb.set("name", os_name_f)?;
    lua.globals().set("os", os_tb)?;

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
    use super::*;

    #[test]
    fn run_test_script() {
        run_file(DATA).unwrap();
    }

    const DATA: &str = r##"
function new_thing()
    local thing = {}
    setmetatable(thing, {
        __close = function()
            print("thing closed")
        end
    })
    return thing
end

do
    local x <close> = new_thing()
    print("using thing")
end

local target_dir = '/tmp/lush-1'
fs.mkdir(target_dir)
print('pwd: ' .. tostring(env.pwd()))
files.zip("/tmp/lush-1/new_post.zip", "src")

env.pushd(target_dir)
local files = fs.ls()
for i = 1, #files do
    print(files[i])
end
env.popd()
fs.rmdir(target_dir, { recursive = true })

env.set('NAME', 'Thiago')
print('ENV: ' .. env.get('NAME'))
env.del('NAME')
print('ENV: ' .. tostring(env.get('NAME')))

print('os name: ' .. os.name())
    "##;
}