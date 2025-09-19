use mlua::Lua;

pub(crate) fn split(_lua: &Lua, (buf, patt, keep_empty): (String, String, Option<bool>)) -> mlua::Result<Vec<String>> {
    let keep_empty = keep_empty.unwrap_or(true);
    let items: Vec<String> = buf.split(&patt)
        .filter(|item| keep_empty || !item.is_empty())
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    Ok(items)
}

pub(crate) fn startswith(_lua: &Lua, (buf, patt): (String, String)) -> mlua::Result<bool> {
    let res = buf.starts_with(&patt);
    Ok(res)
}

pub(crate) fn endswith(_lua: &Lua, (buf, patt): (String, String)) -> mlua::Result<bool> {
    let res = buf.ends_with(&patt);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_split_keep_empty_true() {
        let lua = Lua::new();
        let sep = ",".to_string();

        // Basic case
        let res = split(&lua, ("a,b,c".to_string(), sep.clone(), Some(true))).unwrap();
        assert_eq!(res, vec!["a", "b", "c"]);

        // Consecutive delimiters
        let res = split(&lua, ("a,,b".to_string(), sep.clone(), Some(true))).unwrap();
        assert_eq!(res, vec!["a", "", "b"]);

        // Leading delimiter
        let res = split(&lua, (",a,b".to_string(), sep.clone(), Some(true))).unwrap();
        assert_eq!(res, vec!["", "a", "b"]);

        // Trailing delimiter
        let res = split(&lua, ("a,b,".to_string(), sep.clone(), Some(true))).unwrap();
        assert_eq!(res, vec!["a", "b", ""]);

        // Only delimiters
        let res = split(&lua, (",,,".to_string(), sep.clone(), Some(true))).unwrap();
        assert_eq!(res, vec!["", "", "", ""]);

        // No delimiter found
        let res = split(&lua, ("abc".to_string(), sep.clone(), Some(true))).unwrap();
        assert_eq!(res, vec!["abc"]);

        // Empty input
        let res = split(&lua, ("".to_string(), sep.clone(), Some(true))).unwrap();
        assert_eq!(res, vec![""]);
    }

    #[test]
    fn test_split_keep_empty_false() {
        let lua = Lua::new();
        let sep = ",".to_string();

        // Basic case
        let res = split(&lua, ("a,b,c".to_string(), sep.clone(), Some(false))).unwrap();
        assert_eq!(res, vec!["a", "b", "c"]);

        // Consecutive delimiters
        let res = split(&lua, ("a,,b".to_string(), sep.clone(), Some(false))).unwrap();
        assert_eq!(res, vec!["a", "b"]);

        // Leading delimiter
        let res = split(&lua, (",a,b".to_string(), sep.clone(), Some(false))).unwrap();
        assert_eq!(res, vec!["a", "b"]);

        // Trailing delimiter
        let res = split(&lua, ("a,b,".to_string(), sep.clone(), Some(false))).unwrap();
        assert_eq!(res, vec!["a", "b"]);

        // Only delimiters
        let res = split(&lua, (",,,".to_string(), sep.clone(), Some(false))).unwrap();
        assert_eq!(res, Vec::<String>::new());

        // No delimiter found
        let res = split(&lua, ("abc".to_string(), sep.clone(), Some(false))).unwrap();
        assert_eq!(res, vec!["abc"]);

        // Empty input
        let res = split(&lua, ("".to_string(), sep.clone(), Some(false))).unwrap();
        assert_eq!(res, Vec::<String>::new());
    }

    #[test]
    fn test_split_keep_empty_default() {
        let lua = Lua::new();
        let sep = ",".to_string();

        // When keep_empty is None, default is true
        let res = split(&lua, ("a,,b".to_string(), sep.clone(), None)).unwrap();
        assert_eq!(res, vec!["a", "", "b"]);
    }

    #[test]
    fn test_startswith() {
        let lua = Lua::new();

        // Positive cases
        assert_eq!(startswith(&lua, ("hello world".to_string(), "hello".to_string())).unwrap(), true);
        assert_eq!(startswith(&lua, ("rustacean".to_string(), "rust".to_string())).unwrap(), true);

        // Negative cases
        assert_eq!(startswith(&lua, ("hello world".to_string(), "world".to_string())).unwrap(), false);
        assert_eq!(startswith(&lua, ("abc".to_string(), "abcd".to_string())).unwrap(), false);

        // Edge cases
        assert_eq!(startswith(&lua, ("".to_string(), "".to_string())).unwrap(), true);
        assert_eq!(startswith(&lua, ("hello".to_string(), "".to_string())).unwrap(), true);
        assert_eq!(startswith(&lua, ("".to_string(), "hello".to_string())).unwrap(), false);
    }

    #[test]
    fn test_endswith() {
        let lua = Lua::new();

        // Positive cases
        assert_eq!(endswith(&lua, ("hello world".to_string(), "world".to_string())).unwrap(), true);
        assert_eq!(endswith(&lua, ("rustacean".to_string(), "cean".to_string())).unwrap(), true);

        // Negative cases
        assert_eq!(endswith(&lua, ("hello world".to_string(), "hello".to_string())).unwrap(), false);
        assert_eq!(endswith(&lua, ("abc".to_string(), "z".to_string())).unwrap(), false);

        // Edge cases
        assert_eq!(endswith(&lua, ("".to_string(), "".to_string())).unwrap(), true);
        assert_eq!(endswith(&lua, ("hello".to_string(), "".to_string())).unwrap(), true);
        assert_eq!(endswith(&lua, ("".to_string(), "hello".to_string())).unwrap(), false);
    }
}
