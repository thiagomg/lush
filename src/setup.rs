use std::path::PathBuf;
use mlua::Lua;
use mlua::prelude::LuaResult;
use crate::modules::pipeline_exec::*;
use crate::modules::files::*;
use crate::modules::environment::*;
use crate::modules::filesystem::*;
use crate::modules::net::*;
use crate::modules::os::*;
use crate::modules::path::*;
use crate::modules::string::{endswith, split, startswith};
use crate::modules::toml::load_file as load_toml;
use crate::modules::toml::from_string as from_string_toml;
use crate::modules::toml::save_file as save_toml;
use crate::modules::json::load_file as load_json;
use crate::modules::json::from_string as from_string_json;
use crate::modules::json::save_file as save_json;
use crate::preprocessor::{interpolate_strings, replace_shell_exec, replace_sub_shell};

pub(crate) struct LushContext {
    pub dir_stack: Vec<PathBuf>,
}

pub(crate) fn set_utils(lua: &Lua) -> LuaResult<()> {
    // Env
    let env_tb = lua.create_table()?;
    env_tb.set("cd", lua.create_function(chdir)?)?;
    env_tb.set("pushd", lua.create_function_mut(pushd)?)?;
    env_tb.set("popd", lua.create_function_mut(popd)?)?;
    env_tb.set("pwd", lua.create_function(pwd)?)?;
    env_tb.set("get", lua.create_function(get_env)?)?;
    env_tb.set("set", lua.create_function(set_env)?)?;
    env_tb.set("del", lua.create_function(rem_env)?)?;
    env_tb.set("print", lua.create_function(print)?)?;
    env_tb.set("cwd", lua.create_function(cwd)?)?;
    lua.globals().set("env", env_tb)?;

    // File System
    let filesystem_tb = lua.create_table()?;
    filesystem_tb.set("ls", lua.create_function(ls)?)?;
    filesystem_tb.set("mkdir", lua.create_function(mkdir)?)?;
    filesystem_tb.set("rmdir", lua.create_function(rmdir)?)?;
    filesystem_tb.set("copy", lua.create_function(copy_file)?)?;
    filesystem_tb.set("move", lua.create_function(move_file)?)?;
    filesystem_tb.set("rm", lua.create_function(delete_file)?)?;
    filesystem_tb.set("exists", lua.create_function(file_exists)?)?;
    filesystem_tb.set("is_dir", lua.create_function(is_dir)?)?;
    filesystem_tb.set("is_file", lua.create_function(is_file)?)?;
    filesystem_tb.set("parent", lua.create_function(parent)?)?;
    filesystem_tb.set("read_file", lua.create_function(read_file)?)?;
    filesystem_tb.set("write_file", lua.create_function(write_file)?)?;
    lua.globals().set("fs", filesystem_tb)?;

    // Operating System
    let os_tb: mlua::Table = lua.globals().get("os")?;
    os_tb.set("name", lua.create_function(os_name)?)?;
    os_tb.set("proc_names", lua.create_function(proc_names)?)?;
    os_tb.set("proc_exes", lua.create_function(proc_exes)?)?;
    os_tb.set("pipe_exec", lua.create_function(run_exec)?)?;
    os_tb.set("pipeline", lua.create_function(run_pipe)?)?;
    os_tb.set("mkdtemp", lua.create_function(mkdtemp)?)?;

    // Compression
    let files_tb = lua.create_table()?;
    files_tb.set("zip", lua.create_function(create_zip)?)?;
    files_tb.set("unzip", lua.create_function(extract_zip)?)?;
    files_tb.set("compress", lua.create_function(compress)?)?;
    files_tb.set("decompress", lua.create_function(decompress)?)?;
    lua.globals().set("files", files_tb)?;

    let net_tb = lua.create_table()?;
    net_tb.set("wget", lua.create_function(wget)?)?;
    lua.globals().set("net", net_tb)?;

    let toml_tb = lua.create_table()?;
    toml_tb.set("load_file", lua.create_function(load_toml)?)?;
    toml_tb.set("from_string", lua.create_function(from_string_toml)?)?;
    toml_tb.set("save_file", lua.create_function(save_toml)?)?;
    lua.globals().set("toml", toml_tb)?;

    let json_tb = lua.create_table()?;
    json_tb.set("load_file", lua.create_function(load_json)?)?;
    json_tb.set("from_string", lua.create_function(from_string_json)?)?;
    json_tb.set("save_file", lua.create_function(save_json)?)?;
    lua.globals().set("json", json_tb)?;

    let path_tb = lua.create_table()?;
    path_tb.set("join", lua.create_function(path_join)?)?;
    lua.globals().set("path", path_tb)?;

    let string_tb: mlua::Table = lua.globals().get("string")?;
    string_tb.set("split", lua.create_function(split)?)?;
    string_tb.set("startswith", lua.create_function(startswith)?)?;
    string_tb.set("endswith", lua.create_function(endswith)?)?;
    lua.globals().set("string", string_tb)?;

    Ok(())
}

pub(crate) fn run_script(script: &str, input_file: PathBuf, args: Vec<String>) -> LuaResult<()> {
    let lua = Lua::new();
    let ctx = LushContext {
        dir_stack: vec![],
    };
    lua.set_app_data(ctx);
    set_utils(&lua)?;

    let script_file_name = input_file.to_str().unwrap().to_string();
    let mut full_args = vec![script_file_name.clone()];
    full_args.extend(args);

    // Build the Lua `arg` table
    lua.globals().set("arg", lua.create_table_from(
        full_args.iter().enumerate().map(|(i, arg)| (i, arg.clone()))
    )?)?;

    // Adding the script directory to the package path to simplify module loading
    let script_dir = input_file.parent().unwrap().to_str().unwrap();
    let add_path = format!(r#"package.path = "{script_dir}/?.lua;{script_dir}/?.lush;" .. package.path"#);
    lua.load(&add_path).exec()?;

    // Before loading the script, let's run through pre-processors
    let script = replace_shell_exec(&script);
    let script = replace_sub_shell(&script);
    let script = interpolate_strings(&script);

    lua.load(script).set_name(script_file_name).exec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::data::DATA;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn run_test_script() {
        run_script(DATA, PathBuf::from("script.lua"), vec![]).unwrap();
    }

    #[test]
    fn run_builtin_os() {
        run_script("os.execute('ls')", PathBuf::from("script.lua"), vec![]).unwrap();
    }

    #[test]
    fn invalid_function() {
        let data = r##"

        env.invalid_func(1)
        "##;
        let res = run_script(data, PathBuf::from("script.lua"), vec![]).unwrap_err();

        let ts = res.to_string();
        println!("{}", ts);
    }

    #[test]
    fn test_toml_from_string() {
        let script = r#"
        local toml_str = [[
[server]
host = "localhost"
port = 8080
enabled = true

[database]
name = "mydb"
timeout = 30
        ]]

        local data = toml.from_string(toml_str)
        assert(data.server.host == "localhost")
        assert(data.server.port == 8080)
        assert(data.server.enabled == true)
        assert(data.database.name == "mydb")
        assert(data.database.timeout == 30)
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_toml_from_string_invalid() {
        let script = r#"
        local invalid_toml = "invalid toml content ["
        local success, err = pcall(function()
            toml.from_string(invalid_toml)
        end)
        assert(not success)
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_toml_load_and_save_file() {
        let temp_dir = TempDir::new().unwrap();
        let toml_file = temp_dir.path().join("test.toml");

        // Create initial TOML file
        let toml_content = r#"
[app]
name = "test_app"
version = "1.0.0"
debug = false

[features]
logging = true
metrics = false
        "#;
        fs::write(&toml_file, toml_content).unwrap();

        let script = format!(r#"
        -- Load the TOML file
        local data = toml.load_file("{}")
        assert(data.app.name == "test_app")
        assert(data.app.version == "1.0.0")
        assert(data.app.debug == false)
        assert(data.features.logging == true)
        assert(data.features.metrics == false)

        -- Modify the data
        data.app.version = "2.0.0"
        data.app.debug = true
        data.features.new_feature = "added"

        -- Save it back
        toml.save_file("{}", data)

        -- Load again to verify changes
        local updated_data = toml.load_file("{}")
        assert(updated_data.app.version == "2.0.0")
        assert(updated_data.app.debug == true)
        assert(updated_data.features.new_feature == "added")
        "#,
        toml_file.display(),
        toml_file.display(),
        toml_file.display());

        run_script(&script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_toml_load_file_nonexistent() {
        let script = r#"
        local success, err = pcall(function()
            toml.load_file("/nonexistent/file.toml")
        end)
        assert(not success)
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_json_from_string() {
        let script = r#"
        local json_str = [[{
            "name": "John Doe",
            "age": 30,
            "active": true,
            "balance": 123.45,
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            },
            "hobbies": ["reading", "swimming", "coding"]
        }]]

        local data = json.from_string(json_str)
        assert(data.name == "John Doe")
        assert(data.age == 30)
        assert(data.active == true)
        assert(data.balance == 123.45)
        assert(data.address.street == "123 Main St")
        assert(data.address.city == "Anytown")
        assert(#data.hobbies == 3)
        assert(data.hobbies[1] == "reading")
        assert(data.hobbies[2] == "swimming")
        assert(data.hobbies[3] == "coding")
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_json_from_string_invalid() {
        let script = r#"
        local invalid_json = '{"invalid": json content'
        local success, err = pcall(function()
            json.from_string(invalid_json)
        end)
        assert(not success)
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_json_load_and_save_file() {
        let temp_dir = TempDir::new().unwrap();
        let json_file = temp_dir.path().join("test.json");

        // Create initial JSON file
        let json_content = r#"{
            "config": {
                "host": "example.com",
                "port": 3000,
                "ssl": true
            },
            "users": ["alice", "bob", "charlie"],
            "settings": {
                "theme": "dark",
                "notifications": false
            }
        }"#;
        fs::write(&json_file, json_content).unwrap();

        let script = format!(r#"
        -- Load the JSON file
        local data = json.load_file("{}")
        assert(data.config.host == "example.com")
        assert(data.config.port == 3000)
        assert(data.config.ssl == true)
        assert(#data.users == 3)
        assert(data.users[1] == "alice")
        assert(data.settings.theme == "dark")
        assert(data.settings.notifications == false)

        -- Modify the data
        data.config.port = 4000
        data.config.ssl = false
        data.users[4] = "david"
        data.settings.new_setting = "value"

        -- Save it back
        json.save_file("{}", data)

        -- Load again to verify changes
        local updated_data = json.load_file("{}")
        assert(updated_data.config.port == 4000)
        assert(updated_data.config.ssl == false)
        assert(#updated_data.users == 4)
        assert(updated_data.users[4] == "david")
        assert(updated_data.settings.new_setting == "value")
        "#,
        json_file.display(),
        json_file.display(),
        json_file.display());

        run_script(&script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_json_load_file_nonexistent() {
        let script = r#"
        local success, err = pcall(function()
            json.load_file("/nonexistent/file.json")
        end)
        assert(not success)
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_json_array_handling() {
        let script = r#"
        local json_array = '[1, 2, 3, "hello", true, null]'
        local data = json.from_string(json_array)
        -- 5 as nil will change the result of array size
        assert(#data == 5)
        assert(data[1] == 1)
        assert(data[2] == 2)
        assert(data[3] == 3)
        assert(data[4] == "hello")
        assert(data[5] == true)
        assert(data[6] == nil) -- JSON null becomes Lua nil
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_json_nested_objects() {
        let script = r#"
        local nested_json = [[{
            "level1": {
                "level2": {
                    "level3": {
                        "value": "deep_value",
                        "number": 42
                    }
                }
            }
        }]]

        local data = json.from_string(nested_json)
        assert(data.level1.level2.level3.value == "deep_value")
        assert(data.level1.level2.level3.number == 42)
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_toml_array_handling() {
        let script = r#"
        local toml_str = [=[
numbers = [1, 2, 3]
strings = ["a", "b", "c"]
mixed = [1, "two", true]

[[items]]
name = "first"
value = 10

[[items]]
name = "second"
value = 20
]=]

        local data = toml.from_string(toml_str)
        assert(#data.numbers == 3)
        assert(data.numbers[1] == 1)
        assert(#data.strings == 3)
        assert(data.strings[1] == "a")
        assert(data.mixed[3] == true)
        assert(#data.items == 2)
        assert(data.items[1].name == "first")
        assert(data.items[2].value == 20)
        "#;

        run_script(script, PathBuf::from("test.lua"), vec![]).unwrap();
    }

    #[test]
    fn test_roundtrip_data_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let toml_file = temp_dir.path().join("roundtrip.toml");
        let json_file = temp_dir.path().join("roundtrip.json");

        let script = format!(r#"
        -- Create test data
        local test_data = {{
            string_val = "hello world",
            number_val = 123,
            float_val = 45.67,
            bool_val = true,
            nested = {{
                inner_string = "nested value",
                inner_number = 999
            }}
        }}

        -- Test TOML roundtrip
        toml.save_file("{}", test_data)
        local toml_loaded = toml.load_file("{}")
        assert(toml_loaded.string_val == test_data.string_val)
        assert(toml_loaded.number_val == test_data.number_val)
        assert(toml_loaded.bool_val == test_data.bool_val)
        assert(toml_loaded.nested.inner_string == test_data.nested.inner_string)

        -- Test JSON roundtrip
        json.save_file("{}", test_data)
        local json_loaded = json.load_file("{}")
        assert(json_loaded.string_val == test_data.string_val)
        assert(json_loaded.number_val == test_data.number_val)
        assert(json_loaded.bool_val == test_data.bool_val)
        assert(json_loaded.nested.inner_string == test_data.nested.inner_string)
        "#,
        toml_file.display(),
        toml_file.display(),
        json_file.display(),
        json_file.display());

        run_script(&script, PathBuf::from("test.lua"), vec![]).unwrap();
    }
}
