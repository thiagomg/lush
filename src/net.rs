use std::fs::File;
use std::io;
use mlua::Lua;
use mlua::prelude::LuaError;
use reqwest::blocking::Client;

pub fn wget(_lua: &Lua, (url, output_file): (String, Option<String>)) -> mlua::Result<String> {

    let client = Client::new();
    let mut response = match client.get(&url).send() {
        Ok(response) => response,
        Err(e) => return Err(LuaError::external(e)),
    };

    let filename = output_file.unwrap_or_else(|| {
        url.split('/').last().unwrap_or("index.html").to_string()
    });

    let mut file = File::create(&filename)?;
    io::copy(&mut response, &mut file)?;

    Ok(filename)
}
