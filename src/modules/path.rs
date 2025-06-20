use std::path::PathBuf;
use mlua::{Lua, Variadic};

pub(crate) fn path_join(_lua: &Lua, paths: Variadic<String>) -> mlua::Result<String> {
    let mut full_path = PathBuf::new();

    for path in paths.iter() {
        full_path.push(path);
    }

    let str_path = match full_path.to_str() {
        None => return Ok(String::new()),
        Some(x) => x,
    };
    Ok(str_path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{Lua, Variadic};

    #[test]
    fn joins_multiple_paths() {
        let lua = Lua::new();
        let input = Variadic::from(vec![
            String::from("folder"),
            String::from("subfolder"),
            String::from("file.txt"),
        ]);
        let result = path_join(&lua, input).unwrap();
        #[cfg(unix)]
        assert_eq!(result, "folder/subfolder/file.txt");
        #[cfg(windows)]
        assert_eq!(result, "folder\\subfolder\\file.txt");
    }

    #[test]
    fn handles_empty_input() {
        let lua = Lua::new();
        let input = Variadic::from(vec![]);
        let result = path_join(&lua, input).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn handles_absolute_paths() {
        let lua = Lua::new();
        let input = Variadic::from(vec![
            String::from("/root"),
            String::from("etc"),
            String::from("config"),
        ]);
        let result = path_join(&lua, input).unwrap();
        #[cfg(unix)]
        assert_eq!(result, "/root/etc/config");
    }

    #[test]
    fn handles_trailing_separators() {
        let lua = Lua::new();
        let input = Variadic::from(vec![
            String::from("folder/"),
            String::from("sub/"),
            String::from("file"),
        ]);
        let result = path_join(&lua, input).unwrap();
        #[cfg(unix)]
        assert_eq!(result, "folder/sub/file");
        #[cfg(windows)]
        assert_eq!(result, "folder\\sub\\file");
    }

    #[test]
    fn result_is_utf8() {
        let lua = Lua::new();
        let input = Variadic::from(vec![String::from("路径"), String::from("文件.txt")]);
        let result = path_join(&lua, input).unwrap();
        #[cfg(unix)]
        assert_eq!(result, "路径/文件.txt");
        #[cfg(windows)]
        assert_eq!(result, "路径\\文件.txt");
    }
}
