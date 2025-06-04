use std::fs;
use mlua::Lua;

pub(crate) fn load_file(lua: &Lua, path: String) -> mlua::Result<mlua::Value> {
    let content = fs::read_to_string(path).map_err(|e| mlua::Error::external(e))?;
    let parsed: toml::Value = toml::from_str(&content).map_err(|e| mlua::Error::external(e))?;
    convert_toml_to_lua(lua, parsed)
}

pub(crate) fn save_file(_lua: &Lua, (path, table): (String, mlua::Value)) -> mlua::Result<()> {
    let toml_value = convert_lua_to_toml(&table)?;
    let toml_str = toml::to_string_pretty(&toml_value).map_err(mlua::Error::external)?;
    fs::write(path, toml_str).map_err(mlua::Error::external)?;
    
    Ok(())
}

fn convert_toml_to_lua(lua: &Lua, value: toml::Value) -> mlua::Result<mlua::Value> {
    match value {
        toml::Value::String(s) => Ok(mlua::Value::String(lua.create_string(&s)?)),
        toml::Value::Integer(i) => Ok(mlua::Value::Integer(i)),
        toml::Value::Float(f) => Ok(mlua::Value::Number(f)),
        toml::Value::Boolean(b) => Ok(mlua::Value::Boolean(b)),
        toml::Value::Datetime(dt) => Ok(mlua::Value::String(lua.create_string(&dt.to_string())?)),
        toml::Value::Array(arr) => {
            let lua_table = lua.create_table()?;
            for (i, item) in arr.into_iter().enumerate() {
                lua_table.set(i + 1, convert_toml_to_lua(lua, item)?)?;
            }
            Ok(mlua::Value::Table(lua_table))
        }
        toml::Value::Table(map) => {
            let lua_table = lua.create_table()?;
            for (k, v) in map {
                lua_table.set(k, convert_toml_to_lua(lua, v)?)?;
            }
            Ok(mlua::Value::Table(lua_table))
        }
    }
}

fn convert_lua_to_toml(value: &mlua::Value) -> mlua::Result<toml::Value> {
    match value {
        mlua::Value::Nil => Err(mlua::Error::FromLuaConversionError {
            from: "nil",
            to: "toml::Value".to_string(),
            message: Some("Cannot convert nil to toml".into()),
        }),
        mlua::Value::Boolean(b) => Ok(toml::Value::Boolean(*b)),
        mlua::Value::Integer(i) => Ok(toml::Value::Integer(*i)),
        mlua::Value::Number(n) => Ok(toml::Value::Float(*n)),
        mlua::Value::String(s) => Ok(toml::Value::String(s.to_str()?.to_string())),
        mlua::Value::Table(t) => {
            // Check if this is an array (integer keys starting from 1)
            let is_array = t.clone().sequence_values::<mlua::Value>().next().is_some();
            if is_array {
                let mut arr = Vec::new();
                for pair in t.clone().sequence_values::<mlua::Value>() {
                    arr.push(convert_lua_to_toml(&pair?)?);
                }
                Ok(toml::Value::Array(arr))
            } else {
                let mut map = toml::map::Map::new();
                for pair in t.clone().pairs::<mlua::Value, mlua::Value>() {
                    let (k, v) = pair?;
                    let k_str = match k {
                        mlua::Value::String(s) => s.to_str()?.to_string(),
                        mlua::Value::Integer(i) => i.to_string(),
                        mlua::Value::Number(n) => n.to_string(),
                        _ => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: "table key",
                                to: "String".to_string(),
                                message: Some("Only string or number keys are supported".into()),
                            })
                        }
                    };
                    map.insert(k_str, convert_lua_to_toml(&v)?);
                }
                Ok(toml::Value::Table(map))
            }
        }
        _ => Err(mlua::Error::FromLuaConversionError {
            from: "unsupported type",
            to: "toml::Value".to_string(),
            message: Some("Unsupported Lua value".into()),
        }),
    }
}
