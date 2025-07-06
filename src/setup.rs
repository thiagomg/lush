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
use crate::modules::string::split;
use crate::modules::toml::load_file as load_toml;
use crate::modules::toml::save_file as save_toml;
use crate::modules::json::load_file as load_json;
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
    toml_tb.set("save_file", lua.create_function(save_toml)?)?;
    lua.globals().set("toml", toml_tb)?;

    let json_tb = lua.create_table()?;
    json_tb.set("load_file", lua.create_function(load_json)?)?;
    json_tb.set("save_file", lua.create_function(save_json)?)?;
    lua.globals().set("json", json_tb)?;

    let path_tb = lua.create_table()?;
    path_tb.set("join", lua.create_function(path_join)?)?;
    lua.globals().set("path", path_tb)?;

    let string_tb: mlua::Table = lua.globals().get("string")?;
    string_tb.set("split", lua.create_function(split)?)?;
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
    let script = interpolate_strings(script);
    let script = replace_shell_exec(&script);
    let script = replace_sub_shell(&script);
    
    lua.load(script).set_name(script_file_name).exec()
}

#[cfg(test)]
mod tests {
    use crate::test::data::DATA;
    use super::*;

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
    
    
}