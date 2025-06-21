use std::collections::HashMap;
use std::fmt::{self, Display, Write};

#[derive(Debug)]
pub enum FormatError {
    InvalidPlaceholder(String),
    MissingArgument(String),
    WriteError(fmt::Error),
}

impl Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::InvalidPlaceholder(s) => write!(f, "Invalid placeholder: {}", s),
            FormatError::MissingArgument(s) => write!(f, "Missing argument: {}", s),
            FormatError::WriteError(e) => write!(f, "Write error: {}", e),
        }
    }
}

impl std::error::Error for FormatError {}

pub struct FormatArgs {
    positional: Vec<Box<dyn Display>>,
    named: HashMap<String, Box<dyn Display>>,
}

impl FormatArgs {
    pub fn new() -> Self {
        Self {
            positional: Vec::new(),
            named: HashMap::new(),
        }
    }

    pub fn add_positional<T: Display + 'static>(mut self, value: T) -> Self {
        self.positional.push(Box::new(value));
        self
    }

    pub fn add_named<T: Display + 'static>(mut self, name: impl Into<String>, value: T) -> Self {
        self.named.insert(name.into(), Box::new(value));
        self
    }
}

pub fn dynamic_format(pattern: &str, args: &FormatArgs) -> Result<String, FormatError> {
    let mut result = String::new();
    let mut chars = pattern.chars().peekable();
    let mut auto_index = 1;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                // Escaped brace {{
                chars.next();
                result.push('{');
            } else {
                // Find the closing brace
                let mut placeholder = String::new();
                let mut found_closing = false;

                for inner_ch in chars.by_ref() {
                    if inner_ch == '}' {
                        found_closing = true;
                        break;
                    }
                    placeholder.push(inner_ch);
                }

                if !found_closing {
                    return Err(FormatError::InvalidPlaceholder(format!("{{{}", placeholder)));
                }

                // Handle different placeholder types
                if placeholder.is_empty() {
                    // Simple {} placeholder - use auto-incrementing index. Using -1 as lua starts with 1
                    if (auto_index - 1) >= args.positional.len() {
                        return Err(FormatError::MissingArgument(format!("positional argument {}", auto_index)));
                    }
                    write!(&mut result, "{}", args.positional[auto_index - 1])
                        .map_err(FormatError::WriteError)?;
                    auto_index += 1;
                } else if let Ok(index) = placeholder.parse::<usize>() {
                    // Numeric {0}, {1}, etc. placeholder. Using -1 as lua starts with 1
                    if (index - 1) >= args.positional.len() {
                        return Err(FormatError::MissingArgument(format!("positional argument {}", index)));
                    }
                    write!(&mut result, "{}", args.positional[index - 1])
                        .map_err(FormatError::WriteError)?;
                } else {
                    // Named placeholder {tag-name}
                    match args.named.get(&placeholder) {
                        Some(value) => {
                            write!(&mut result, "{}", value)
                                .map_err(FormatError::WriteError)?;
                        }
                        None => {
                            return Err(FormatError::MissingArgument(format!("named argument '{}'", placeholder)));
                        }
                    }
                }
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                // Escaped brace }}
                chars.next();
                result.push('}');
            } else {
                // Unmatched closing brace
                return Err(FormatError::InvalidPlaceholder("}".to_string()));
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

// Convenience macro for easier usage
#[macro_export]
macro_rules! dyn_format {
    // Pattern: dyn_format!("pattern", arg1, arg2; "name1" => val1, "name2" => val2)
    ($pattern:expr, $($pos_arg:expr),*; $($name:literal => $named_arg:expr),*) => {
        {
            let mut args = FormatArgs::new();
            $(
                args = args.add_positional($pos_arg);
            )*
            $(
                args = args.add_named($name, $named_arg);
            )*
            dynamic_format($pattern, &args)
        }
    };
    // Pattern: dyn_format!("pattern", arg1, arg2) - only positional
    ($pattern:expr, $($pos_arg:expr),*) => {
        {
            let mut args = FormatArgs::new();
            $(
                args = args.add_positional($pos_arg);
            )*
            dynamic_format($pattern, &args)
        }
    };
    // Pattern: dyn_format!("pattern"; "name1" => val1, "name2" => val2) - only named
    ($pattern:expr; $($name:literal => $named_arg:expr),*) => {
        {
            let mut args = FormatArgs::new();
            $(
                args = args.add_named($name, $named_arg);
            )*
            dynamic_format($pattern, &args)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positional_auto() {
        let args = FormatArgs::new()
            .add_positional("world")
            .add_positional(42);
        let result = dynamic_format("Hello, {}! Number: {}", &args).unwrap();
        assert_eq!(result, "Hello, world! Number: 42");
    }

    #[test]
    fn test_positional_explicit() {
        let args = FormatArgs::new()
            .add_positional("Alice")
            .add_positional("Bob");
        let result = dynamic_format("Hello, {2} and {1}!", &args).unwrap();
        assert_eq!(result, "Hello, Bob and Alice!");
    }

    #[test]
    fn test_named() {
        let args = FormatArgs::new()
            .add_named("name", "Alice")
            .add_named("user-id", 123);
        let result = dynamic_format("Hello, {name}! Your ID is {user-id}.", &args).unwrap();
        assert_eq!(result, "Hello, Alice! Your ID is 123.");
    }

    #[test]
    fn test_mixed() {
        let args = FormatArgs::new()
            .add_positional("Alice")
            .add_positional(30)
            .add_named("city", "New York")
            .add_named("user-type", "premium");
        let result = dynamic_format(
            "Hello, {}! You are {} years old, living in {city} as a {user-type} user.",
            &args
        ).unwrap();
        assert_eq!(result, "Hello, Alice! You are 30 years old, living in New York as a premium user.");
    }

    #[test]
    fn test_escaped_braces() {
        let args = FormatArgs::new().add_positional("value");
        let result = dynamic_format("Use {{}} for braces, {} for args", &args).unwrap();
        assert_eq!(result, "Use {} for braces, value for args");
    }

    #[test]
    fn test_macro_positional_only() {
        let result = dyn_format!("Hello, {} and {}!", "Alice", "Bob").unwrap();
        assert_eq!(result, "Hello, Alice and Bob!");
    }

    #[test]
    fn test_macro_named_only() {
        let result = dyn_format!("Hello, {name}! Your ID is {user-id}.";
            "name" => "Alice",
            "user-id" => 123
        ).unwrap();
        assert_eq!(result, "Hello, Alice! Your ID is 123.");
    }

    #[test]
    fn test_macro_mixed() {
        let result = dyn_format!(
            "Hello, {}! You are {} years old, living in {city} as a {user-type} user.",
            "Alice", 30;
            "city" => "New York",
            "user-type" => "premium"
        ).unwrap();
        assert_eq!(result, "Hello, Alice! You are 30 years old, living in New York as a premium user.");
    }

    #[test]
    fn test_positional_explicit_with_macro() {
        let result = dyn_format!("Hello, {2} and {1}!", "Alice", "Bob").unwrap();
        assert_eq!(result, "Hello, Bob and Alice!");
    }

    #[test]
    fn test_missing_positional() {
        let args = FormatArgs::new();
        let result = dynamic_format("Hello, {}!", &args);
        assert!(matches!(result, Err(FormatError::MissingArgument(_))));
    }

    #[test]
    fn test_missing_named() {
        let args = FormatArgs::new();
        let result = dynamic_format("Hello, {name}!", &args);
        assert!(matches!(result, Err(FormatError::MissingArgument(_))));
    }

    #[test]
    fn test_hyphenated_names() {
        let args = FormatArgs::new()
            .add_named("first-name", "Alice")
            .add_named("last-name", "Smith");
        let result = dynamic_format("Hello, {first-name} {last-name}!", &args).unwrap();
        assert_eq!(result, "Hello, Alice Smith!");
    }
}