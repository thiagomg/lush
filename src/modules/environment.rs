use std::env;
use mlua::{Lua, Value, Variadic};
use crate::setup::LushContext;
use crate::utils::dyn_format::{dynamic_format, FormatArgs};

/// Changes the current working directory and pushes the previous one onto the stack.
///
/// This function works by changing the current working directory to `new_dir` and
/// saving the previous directory in `LushContext`'s `dir_stack`.
///
/// # Arguments
///
/// * `lua` - The Lua state in which the application is running.
/// * `new_dir` - The new directory path to switch to.
///
/// # Returns
///
/// * `Ok(())` if the directory is successfully changed.
/// * An error if the directory cannot be changed.
///
/// # Example (in Lua)
///
/// ```lua
/// env.pushd("/some/new/dir")
/// ```
pub(crate) fn pushd(lua: &Lua, new_dir: String) -> mlua::Result<()> {
    let cur_dir = env::current_dir().unwrap();
    env::set_current_dir(new_dir.clone())?;
    if let Some(mut data) = lua.app_data_mut::<LushContext>() {
        data.dir_stack.push(cur_dir);
        // TODO: Print stack optionally
        // let stack: Vec<String> = data.dir_stack.iter().map(|x| x.to_str().unwrap().to_string()).collect();
        // println!("pushd: data.dir_stack: {}", stack.join(", "));
    }
    Ok(())
}

/// Pops the top directory from the stack and changes to it.
///
/// This function restores the directory that was most recently saved in the
/// `dir_stack` of `LushContext`, allowing for a directory stack-like behavior.
///
/// # Arguments
///
/// * `lua` - The Lua state in which the application is running.
///
/// # Returns
///
/// * `Ok(())` if the directory is successfully changed.
/// * An error if there is no directory on the stack or the change fails.
///
/// # Example (in Lua)
///
/// ```lua
/// env.popd()
/// ```
pub(crate) fn popd(lua: &Lua, _: ()) -> mlua::Result<()> {
    if let Some(mut data) = lua.app_data_mut::<LushContext>() {
        if let Some(last_dir) = data.dir_stack.pop() {
            env::set_current_dir(last_dir)?;
            // TODO: Print stack optionally
            // let stack: Vec<String> = data.dir_stack.iter().map(|x| x.to_str().unwrap().to_string()).collect();
            // println!("popd: data.dir_stack: {}", stack.join(", "));
        }
    }
    Ok(())
}

/// Changes the current working directory.
///
/// This function directly changes the current working directory without affecting
/// any directory stack.
///
/// # Arguments
///
/// * `lua` - The Lua state (not used in this function).
/// * `new_dir` - The new directory to switch to.
///
/// # Returns
///
/// * `Ok(())` if the directory is successfully changed.
/// * An error if the directory change fails.
///
/// # Example (in Lua)
///
/// ```lua
/// env.chdir("/another/dir")
/// ```
pub(crate) fn chdir(_lua: &Lua, new_dir: String) -> mlua::Result<()> {
    env::set_current_dir(new_dir)?;
    Ok(())
}

/// Returns the current working directory.
///
/// This function retrieves the current directory path as a string.
///
/// # Arguments
///
/// * `lua` - The Lua state (not used in this function).
///
/// # Returns
///
/// * `Ok(String)` - The current directory as a string.
///
/// # Example (in Lua)
///
/// ```lua
/// print(env.pwd())
/// ```
pub(crate) fn pwd(_lua: &Lua, _: ()) -> mlua::Result<String> {
    let c = env::current_dir()?.to_str().unwrap().to_string();
    Ok(c)
}

/// Sets an environment variable.
///
/// This function sets the value of the specified environment variable.
///
/// # Arguments
///
/// * `lua` - The Lua state (not used in this function).
/// * `name` - The name of the environment variable.
/// * `value` - The value to set.
///
/// # Example (in Lua)
///
/// ```lua
/// env.set("MY_VAR", "some_value")
/// ```
pub(crate) fn set_env(_lua: &Lua, (name, value): (String, String)) -> mlua::Result<()> {
    unsafe {
        env::set_var(name, value);
    }
    Ok(())
}

/// Gets the value of an environment variable.
///
/// This function retrieves the value of the specified environment variable as a Lua `Value`.
/// If the variable does not exist, it returns `nil`.
///
/// # Arguments
///
/// * `lua` - The Lua state.
/// * `name` - The name of the environment variable.
///
/// # Returns
///
/// * `Value::String` - The value of the environment variable.
/// * `Value::Nil` - If the variable does not exist.
///
/// # Example (in Lua)
///
/// ```lua
/// local value = env.get("MY_VAR")
/// print(value)
/// ```
pub(crate) fn get_env(_lua: &Lua, name: String) -> mlua::Result<Value> {
    let c = match env::var(name) {
        Ok(c) => Value::String(_lua.create_string(c)?),
        Err(_e) => Value::Nil,
    };

    Ok(c)
}

/// Removes an environment variable.
///
/// This function removes the specified environment variable.
///
/// # Arguments
///
/// * `lua` - The Lua state (not used in this function).
/// * `name` - The name of the environment variable to remove.
///
/// # Example (in Lua)
///
/// ```lua
/// env.del("MY_VAR")
/// ```
pub(crate) fn rem_env(_lua: &Lua, name: String) -> mlua::Result<()> {
    unsafe { env::remove_var(name); }
    Ok(())
}

/// Prints Lua `Value` tokens to the standard output.
///
/// This function takes a variadic number of Lua `Value`s, converts them to strings, and prints them.
///
/// # Arguments
///
/// * `lua` - The Lua state.
/// * `tokens` - A variadic list of Lua `Value`s to print.
///
/// # Example (in Lua)
///
/// ```lua
/// env.print("Hello", "World", 123)
/// ```
pub(crate) fn print(lua: &Lua, tokens: Variadic<Value>) -> mlua::Result<()> {
    let tab_count = tokens.iter().filter(|x| x.is_table()).count();
    if tab_count == 1 && tokens.len() == 2 && tokens[0].is_string() {
        match (&tokens[0], &tokens[1]) {
            (Value::String(s), Value::Table(t)) => {
                let s = interpolate_string(lua, s.to_str()?.to_string().as_str(), t)?;
                println!("{}", s);
            }
            _ => println!("{:#?}", tokens),
        };
    } else if tab_count == 1 && is_array(&tokens) {
        print_array(&tokens)
    } else if tab_count > 0 {
        println!("{:#?}", tokens);    
    } else {
        let res: Vec<String> = tokens.iter().map(|x| x.to_string().unwrap().to_string()).collect();
        println!("{}", res.join(" "));
    }
    Ok(())
}

fn print_array(tokens: &Variadic<Value>) {
    if tokens.len() != 1 {
        return;
    }

    let item = tokens.get(0).unwrap();
    if let Value::Table(t) = item {
        let mut exp_idx = 1;
        print!("{}", "{");
        loop {
            let val = match t.get::<Value>(exp_idx) {
                Ok(val) if val.is_nil() => break,
                Ok(val) => val,
                Err(_) => break,
            };
            if exp_idx == 1 {
                print!("{}", val.to_string().unwrap());
            } else {
                print!(", {}", val.to_string().unwrap());
            }
            exp_idx += 1;
        }
        println!("{}", "}");
    }
}

fn is_array(tokens: &Variadic<Value>) -> bool {
    if tokens.len() != 1 {
        return false;
    }

    let item = tokens.get(0).unwrap();
    if let Value::Table(t) = item {
        let max = t.pairs::<Value, Value>().count() + 1;
        for i in 1..max {
            let val: Result<Value, _> = t.get(i);
            if val.is_err() { return false; }
            if val.unwrap().is_nil() { return false; }
        }
        return true;
    }

    false
}

fn interpolate_string(_lua: &Lua, format: &str, tokens: &mlua::Table) -> mlua::Result<String> {
    let mut args = FormatArgs::new();
    let mut idx = 0;
    loop {
        idx += 1;
        let val: Value = tokens.get(idx)?;
        if val.is_nil() {
            break;
        }
        
        let str_val = val.to_string()?.to_string();
        args = args.add_positional(str_val);
    }
    
    for pair in tokens.pairs::<Value, Value>() {
        let (key, value) = pair?;
        if let Value::String(key) = key {
            let str_key = key.to_str()?.to_string();
            let str_val = value.to_string()?.to_string(); 
            args = args.add_named(str_key, str_val);
        }
    }

    let s = dynamic_format(format, &args).unwrap_or_else(|_| panic!("Error formatting formatting {}", format));
    Ok(s)
}

pub(crate) fn cwd(_lua: &Lua, _: ()) -> mlua::Result<Option<String>> {
    let cur_dir = match env::current_dir() {
        Ok(d) => Some(d.to_str().unwrap().to_string()),
        Err(_e) => None,
    };
    Ok(cur_dir)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::setup::run_script;
    use super::*;

    #[test]
    fn interpolate_test() {
        let lua = Lua::new();

        let args = lua.create_table().unwrap();
        args.set(1, "Não").unwrap();
        args.set("nome", "nego").unwrap();

        let ret = interpolate_string(&lua, "e foi {nome} mesmo? {1}", &args).unwrap();
        assert_eq!(ret, "e foi nego mesmo? Não");
    }
    
    #[test]
    fn happy_case_2() {
        run_script(r#"
            local args = {10, 20, name='Thiago'}
            env.print('a={}, b={}, nome={name}', args) 
            "#, 
            PathBuf::from("script.lua"), vec![]).unwrap();
    }

    #[test]
    fn happy_case_3() {
        run_script(r#"
            local args = {[1] = 1, [2] = 10, [3] = 20, name='Thiago'}
            env.print('a={2}, b={3}, nome={name}', args) 
            "#,
            PathBuf::from("script.lua"), vec![]).unwrap();
    }

    #[test]
    fn print_array() {
        run_script(r#"
            local args = {[1] = 1, [2] = 10, [3] = 20, name='Thiago'}
            env.print(args)
            "#,
                   PathBuf::from("script.lua"), vec![]).unwrap();

        run_script(r#"
            local args = {[1] = 1, [2] = 10, [3] = 20}
            env.print(args)
            "#,
                   PathBuf::from("script.lua"), vec![]).unwrap();

        run_script(r#"
            local args = {}
            env.print(args)
            "#,
                   PathBuf::from("script.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_is_array_with_proper_array() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set(1, "a").unwrap();
        table.set(2, "b").unwrap();
        table.set(3, "c").unwrap();

        let tokens = Variadic::from_iter([Value::Table(table)]);
        assert!(is_array(&tokens));
    }

    #[test]
    fn test_is_array_with_non_sequential_keys() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set(1, "a").unwrap();
        table.set(3, "b").unwrap(); // skip index 2

        let tokens = Variadic::from_iter([Value::Table(table)]);
        assert!(!is_array(&tokens));
    }

    #[test]
    fn test_is_array_with_string_key() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set(1, "a").unwrap();
        table.set("key", "b").unwrap(); // string key

        let tokens = Variadic::from_iter([Value::Table(table)]);
        assert!(!is_array(&tokens));
    }

    #[test]
    fn test_is_array_with_empty_table() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();

        let tokens = Variadic::from_iter([Value::Table(table)]);
        // By your logic, empty table should be an array (nothing contradicts sequence)
        assert!(is_array(&tokens));
    }

    #[test]
    fn test_is_array_with_wrong_argument_count() {
        let lua = Lua::new();
        let table1 = lua.create_table().unwrap();
        let table2 = lua.create_table().unwrap();

        let tokens = Variadic::from_iter([Value::Table(table1), Value::Table(table2)]);
        assert!(!is_array(&tokens));

        let empty: Variadic<Value> = Variadic::new();
        assert!(!is_array(&empty));
    }

    #[test]
    fn test_is_array_with_wrong_argument_type() {
        let tokens = Variadic::from_iter([Value::String(Lua::new().create_string("not a table").unwrap())]);
        assert!(!is_array(&tokens));
    }


}