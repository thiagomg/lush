use std::{fs, io};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use mlua::{Lua, Table, Value, Variadic};

// TODO: Parse masks, such as *.md
pub(crate) fn ls(_lua: &Lua, paths: Variadic<Value>) -> mlua::Result<Vec<String>> {
    let mut src_path = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        let mut res = vec![];
        for path in paths.iter() {
            res.push(PathBuf::from(path.to_string()?));
        }
        res
    };

    // TODO: Read more than one
    let src_path = src_path.pop().unwrap();
    let files: Vec<String> = fs::read_dir(&src_path)?.into_iter()
        .map(|f| f.unwrap().path())
        .map(|f| f.to_str().unwrap().to_string())
        .collect();

    Ok(files)
}

pub(crate) fn mkdir(_lua: &Lua, path: String) -> mlua::Result<()> {
    let dir = PathBuf::from(path);
    if !Path::exists(&dir) {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

pub(crate) fn rmdir(_lua: &Lua, (path, options): (String, Option<Table>)) -> mlua::Result<()> {
    let md = fs::metadata(&path)?;
    if !md.is_dir() {
        return Err(io::Error::new(ErrorKind::InvalidInput, "Path is not a directory").into());
    }

    let mut rec: bool = false;
    if let Some(ref tb) = options {
        if tb.contains_key("recursive")? {
            rec = tb.get("recursive")?;
        }
    }

    if rec {
        println!("Deleting recursively");
        fs::remove_dir_all(PathBuf::from(&path))?;
    } else {
        println!("Deleting directory");
        fs::remove_dir(PathBuf::from(&path))?;
    }

    Ok(())
}

pub(crate) fn copy_file(_lua: &Lua, (src, target): (String, String)) -> mlua::Result<()> {
    let src_path = PathBuf::from(&src);
    if !src_path.exists() {
        return Err(mlua::Error::RuntimeError(format!("Invalid source path {}", src)));
    }

    let mut target_path = PathBuf::from(&target);
    if target_path.is_dir() {
        let fname = src_path.file_name().unwrap();
        target_path = target_path.join(fname);
    }

    fs::copy(src_path, target_path)?;

    Ok(())
}

pub(crate) fn move_file(_lua: &Lua, (src, target): (String, String)) -> mlua::Result<()> {
    let src_path = PathBuf::from(&src);
    if !src_path.exists() {
        return Err(mlua::Error::RuntimeError(format!("Invalid source path {}", src)));
    }

    let mut target_path = PathBuf::from(&target);
    if target_path.is_dir() {
        let fname = src_path.file_name().unwrap();
        target_path = target_path.join(fname);
    }

    fs::rename(src_path, target_path)?;

    Ok(())
}

pub(crate) fn file_exists(_lua: &Lua, src: String) -> mlua::Result<bool> {
    let src_path = PathBuf::from(&src);
    Ok(src_path.exists())
}
