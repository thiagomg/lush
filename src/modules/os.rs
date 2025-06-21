use std::{env, path::PathBuf};
use mlua::{Lua, Table};
use sysinfo::System;
use tempfile::tempdir;

use crate::TEMP_PATHS;

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

/// Returns a table containing running process names
///
/// Returns:
///
/// * A table { pid: process_name }
///
/// Example:
///
/// ```lua
/// os.proc_names()
/// -- Returns
/// {
///     1121="watchdogd",
///     80574="periodic-wrapper",
///     1309="distnoted"
/// }
/// ```
pub(crate) fn proc_names(lua: &Lua, _: ()) -> mlua::Result<Table> {
    let sys = System::new_all();

    let tb = lua.create_table()?;
    for (pid, process) in sys.processes() {
        tb.set(pid.as_u32(), process.name().to_str().unwrap())?;
    }

    Ok(tb)
}

/// Returns a table containing running process executables with path
///
/// Returns:
///
/// * A table { pid: process_name }
///
/// Example:
///
/// ```lua
/// os.proc_exes()
/// -- Returns
/// {
///     1121="/usr/libexec/watchdogd",
///     80574="/usr/libexec/periodic-wrapper",
///     1309="/usr/sbin/distnoted"
/// }
/// ```
pub(crate) fn proc_exes(lua: &Lua, _: ()) -> mlua::Result<Table> {
    let sys = System::new_all();

    let tb = lua.create_table()?;
    for (pid, process) in sys.processes() {
        tb.set(pid.as_u32(), process.exe().unwrap().to_str().unwrap())?;
    }

    Ok(tb)
}

pub(crate) fn mkdtemp(_lua: &Lua, _: ()) -> mlua::Result<String> {
    let dir_path: PathBuf = tempdir()?.keep();

    TEMP_PATHS.lock().unwrap().push(dir_path.clone());

    Ok(dir_path.to_str().unwrap_or("").to_string())
}
