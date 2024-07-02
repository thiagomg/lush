use std::env;
use std::path::PathBuf;
use mlua::{Lua, Value};
use crate::setup::LushContext;

pub(crate) fn pushd(lua: &Lua, new_dir: String) -> mlua::Result<()> {
    let cur_dir = env::current_dir().unwrap();
    env::set_current_dir(new_dir.clone())?;
    if let Some(mut data) = lua.app_data_mut::<LushContext>() {
        data.dir_stack.push(cur_dir);
        // TODO: Remove print
        println!("data.dir_stack: {}", data.dir_stack.len());
    }
    Ok(())
}

pub(crate) fn popd(lua: &Lua, _: ()) -> mlua::Result<()> {
    if let Some(mut data) = lua.app_data_mut::<LushContext>() {
        if let Some(last_dir) = data.dir_stack.pop() {
            env::set_current_dir(PathBuf::from(last_dir))?;
            // TODO: Remove print
            println!("data.dir_stack: {}", data.dir_stack.len());
        }
    }
    Ok(())
}

pub(crate) fn chdir(_lua: &Lua, new_dir: String) -> mlua::Result<()> {
    env::set_current_dir(new_dir)?;
    Ok(())
}

pub(crate) fn pwd(_lua: &Lua, _: ()) -> mlua::Result<String> {
    let c = env::current_dir()?.to_str().unwrap().to_string();
    Ok(c)
}

pub(crate) fn set_env(_lua: &Lua, (name, value): (String, String)) -> mlua::Result<()> {
    env::set_var(name, value);
    Ok(())
}

pub(crate) fn get_env(_lua: &Lua, name: String) -> mlua::Result<Value> {
    let c = match env::var(name) {
        Ok(c) => Value::String(_lua.create_string(c)?),
        Err(_e) => Value::Nil,
    };

    Ok(c)
}

pub(crate) fn rem_env(_lua: &Lua, name: String) -> mlua::Result<()> {
    env::remove_var(name);
    Ok(())
}
