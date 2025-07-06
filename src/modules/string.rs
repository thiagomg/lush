use mlua::Lua;

pub(crate) fn split(_lua: &Lua, (buf, patt, keep_empty): (String, String, Option<bool>)) -> mlua::Result<Vec<String>> {
    let keep_empty = keep_empty.unwrap_or(true);
    let items: Vec<String> = buf.split(&patt)
        .filter(|item| keep_empty || !item.is_empty())
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    Ok(items)
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
}
