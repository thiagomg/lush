use mlua::{Lua, LuaSerdeExt};
use serde_json::{Value as JsonValue};
use std::fs;
use mlua::prelude::LuaError;

pub(crate) fn load_file(lua: &Lua, path: String) -> mlua::Result<mlua::Value> {
    let content = fs::read_to_string(&path)?;
    let json: JsonValue = match serde_json::from_str(&content) {
        Ok(val) => val,
        Err(e) => return Err(LuaError::RuntimeError(e.to_string())),
    };
    let lua_value = lua.to_value(&json)?;
    Ok(lua_value)
}

pub(crate) fn save_file(lua: &Lua, (path, table): (String, mlua::Value)) -> mlua::Result<()> {
    let json: JsonValue = lua.from_value(table)?;
    let content = match serde_json::to_string_pretty(&json) {
        Ok(val) => val,
        Err(e) => return Err(LuaError::RuntimeError(e.to_string())),
    };
    fs::write(path, content)?;
    Ok(())
}

