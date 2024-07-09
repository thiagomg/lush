use std::env;
use mlua::Lua;

pub(crate) fn os_name(_lua: &Lua, _: ()) -> mlua::Result<String> {
    // - linux
    // - macos
    // - ios
    // - freebsd
    // - dragonfly
    // - netbsd
    // - openbsd
    // - solaris
    // - android
    // - windows

    Ok(env::consts::OS.to_string())
}