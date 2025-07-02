pub fn remove_shebang(script: String) -> String {
    let mut chars = script.chars().peekable();
    let mut i = 0;

    while let Some(&c) = chars.peek() {
        if !c.is_whitespace() && c != '\r' && c != '\n' {
            // It's not shebang
            if c != '#' {
                return script;
            }
            break;
        }
        chars.next();
        i += 1;
    }

    let before = &script[0..i];
    let after = &script[i..];
    format!("{}-- {}", before, after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_shebang_with_shebang() {
        let script = "#!/usr/bin/env lush\n echo Hello, world!".to_string();
        let result = remove_shebang(script);
        assert_eq!(result, "-- #!/usr/bin/env lush\n echo Hello, world!");
        let script = "#!/usr/bin/env lush\r\n echo Hello, world!".to_string();
        let result = remove_shebang(script);
        assert_eq!(result, "-- #!/usr/bin/env lush\r\n echo Hello, world!");
        let script = "#!/usr/bin/env lush\r echo Hello, world!".to_string();
        let result = remove_shebang(script);
        assert_eq!(result, "-- #!/usr/bin/env lush\r echo Hello, world!");
    }

    #[test]
    fn test_remove_shebang_with_shebang_and_blank_lines() {
        let script = "\n\n#!/usr/bin/env lush\nprint('Hello, world!')".to_string();
        let result = remove_shebang(script);
        assert_eq!(result, "\n\n-- #!/usr/bin/env lush\nprint('Hello, world!')");
    }

    #[test]
    fn test_remove_shebang_with_no_shebang() {
        let script = "echo Hello, world!".to_string();
        let result = remove_shebang(script);
        assert_eq!(result, "echo Hello, world!");
    }

    #[test]
    fn test_remove_shebang_with_blank_lines_no_shebang() {
        let script = "\n\n  echo Hello, world!".to_string();
        let result = remove_shebang(script);
        assert_eq!(result, "\n\n  echo Hello, world!");
    }
}
