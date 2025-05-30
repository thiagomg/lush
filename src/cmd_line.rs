use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Lua file to be interpreted and executed
    pub lua_file: PathBuf,

    /// Arguments to be passed to the script
    pub script_args: Vec<String>,
}
