use std::env;
use mlua::Lua;

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