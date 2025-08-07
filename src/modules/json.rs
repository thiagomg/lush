use mlua::{Lua, LuaSerdeExt};
use serde_json::Value as JsonValue;
use std::fs;
use mlua::prelude::{LuaError, LuaValue};

pub(crate) fn load_file(lua: &Lua, path: String) -> mlua::Result<mlua::Value> {
    let content = fs::read_to_string(&path)?;
    from_string(lua, content)
}

pub(crate) fn from_string(lua: &Lua, content: String) -> mlua::Result<mlua::Value> {
    let json: JsonValue = match serde_json::from_str(&content) {
        Ok(val) => val,
        Err(e) => return Err(LuaError::RuntimeError(e.to_string())),
    };
    let lua_value = json_to_lua(lua, json)?;
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

fn json_to_lua(lua: &Lua, value: JsonValue) -> mlua::Result<LuaValue> {
    match value {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Err(LuaError::RuntimeError("Invalid number format".to_string()))
            }
        }
        JsonValue::String(s) => Ok(LuaValue::String(lua.create_string(&s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, item) in arr.into_iter().enumerate() {
                let lua_item = json_to_lua(lua, item)?;
                table.set(i + 1, lua_item)?; // Lua arrays are 1-indexed
            }
            Ok(LuaValue::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (key, value) in obj {
                let lua_value = json_to_lua(lua, value)?;
                table.set(key, lua_value)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}
