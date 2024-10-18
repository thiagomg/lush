use std::env;
use mlua::{Lua, Table};
use sysinfo::System;

/// Returns the name of the operating system the program is running on.
///
/// This function identifies the operating system and returns its name as a string. Possible values include:
/// - `linux`
/// - `macos`
/// - `ios`
/// - `freebsd`
/// - `dragonfly`
/// - `netbsd`
/// - `openbsd`
/// - `solaris`
/// - `android`
/// - `windows`
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `_` - Ignored argument.
///
/// # Returns
///
/// * A `String` containing the name of the operating system.
///
/// # Example (in Lua)
///
/// ```lua
/// local os_name = os.name()
/// print("Operating System:", os_name)
/// ```
pub(crate) fn os_name(_lua: &Lua, _: ()) -> mlua::Result<String> {
    Ok(env::consts::OS.to_string())
}

pub(crate) fn proc_names(lua: &Lua, _: ()) -> mlua::Result<Table> {
    let sys = System::new_all();

    let tb = lua.create_table()?;
    for (pid, process) in sys.processes() {
        tb.set(pid.as_u32(), process.name().to_str().unwrap())?;
    }

    Ok(tb)
}

pub(crate) fn proc_exes(lua: &Lua, _: ()) -> mlua::Result<Table> {
    let sys = System::new_all();

    let tb = lua.create_table()?;
    for (pid, process) in sys.processes() {
        tb.set(pid.as_u32(), process.exe().unwrap().to_str().unwrap())?;
    }

    Ok(tb)
}

#[cfg(test)]
mod tests {
    use mlua::Lua;
    use crate::os::proc_names;

    #[test]
    fn proc_list_test() {
        let lua = Lua::new();
        let _ = proc_names(&lua, ());
    }
}
