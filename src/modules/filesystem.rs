use std::{fs, io};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use mlua::{Lua, Table, Value, Variadic};

/// Lists the contents of the specified directory or the current directory if no path is provided.
///
/// If a path is provided, it lists the files in that directory. Otherwise, it lists the contents
/// of the current working directory.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `paths` - A variadic list of directory paths to list. Only the first path is used.
///
/// # Returns
///
/// * A vector of strings containing the file paths within the directory.
///
/// # Errors
///
/// * Returns an error if the directory cannot be read.
///
/// # Example (in Lua)
///
/// ```lua
/// local files = fs.ls("/some/directory")
/// print(files)
/// ```
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
    let files: Vec<String> = fs::read_dir(&src_path)?
        .map(|f| f.unwrap().path())
        .map(|f| f.to_str().unwrap().to_string())
        .collect();

    Ok(files)
}

/// Creates a directory at the specified path.
///
/// Recursively creates all parent directories if they do not exist.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `path` - The directory path to create.
///
/// # Returns
///
/// * `Ok(())` if the directory is successfully created.
/// * Returns an error if the directory cannot be created.
///
/// # Example (in Lua)
///
/// ```lua
/// fs.mkdir("/new/directory")
/// ```
pub(crate) fn mkdir(_lua: &Lua, path: String) -> mlua::Result<()> {
    let dir = PathBuf::from(path);
    if !Path::exists(&dir) {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Removes a directory at the specified path.
///
/// Supports recursive deletion through an optional argument.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `path` - The directory path to remove.
/// * `options` - Optional table containing a `recursive` flag. If `true`, the directory and its contents are deleted recursively.
///
/// # Returns
///
/// * `Ok(())` if the directory is successfully removed.
/// * Returns an error if the path is not a directory or the deletion fails.
///
/// # Example (in Lua)
///
/// ```lua
/// fs.rmdir("/some/directory", { recursive = true })
/// ```
pub(crate) fn rmdir(_lua: &Lua, (path, options): (String, Option<Table>)) -> mlua::Result<bool> {
    let md = match fs::metadata(&path) {
        Ok(x) => x,
        Err(_) => return Ok(false),
    };
    if !md.is_dir() {
        return Err(io::Error::new(ErrorKind::InvalidInput, "Path is not a directory").into());
    }

    let mut rec: bool = false;
    if let Some(ref tb) = options {
        if tb.contains_key("recursive")? {
            rec = tb.get("recursive")?;
        }
    }

    let res = if rec {
        // println!("Deleting recursively");
        fs::remove_dir_all(PathBuf::from(&path))
    } else {
        // println!("Deleting directory");
        fs::remove_dir(PathBuf::from(&path))
    };

    Ok(res.is_ok())
}

/// Copies a file from the source path to the target path.
///
/// If the target path is a directory, the file is copied into the directory with its original name.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `src` - The source file path.
/// * `target` - The target file or directory path.
///
/// # Returns
///
/// * `Ok(())` if the file is successfully copied.
/// * Returns an error if the source file does not exist or the copy operation fails.
///
/// # Example (in Lua)
///
/// ```lua
/// fs.copy("/path/to/source", "/path/to/destination")
/// ```
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

/// Moves a file from the source path to the target path.
///
/// If the target path is a directory, the file is moved into the directory with its original name.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `src` - The source file path.
/// * `target` - The target file or directory path.
///
/// # Returns
///
/// * `Ok(())` if the file is successfully moved.
/// * Returns an error if the source file does not exist or the move operation fails.
///
/// # Example (in Lua)
///
/// ```lua
/// fs.move("/path/to/source", "/path/to/destination")
/// ```
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

/// Checks if a file exists at the specified path.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `src` - The file path to check.
///
/// # Returns
///
/// * `true` if the file exists, `false` otherwise.
///
/// # Example (in Lua)
///
/// ```lua
/// local exists = fs.exists("/path/to/file")
/// print(exists)
/// ```
pub(crate) fn file_exists(_lua: &Lua, src: String) -> mlua::Result<bool> {
    let src_path = PathBuf::from(&src);
    Ok(src_path.exists())
}

/// Deletes a file at the specified path.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `path` - The directory path to remove.
/// * `options` - Optional table containing a `recursive` flag. If `true`, the directory and its contents are deleted recursively.
///
/// # Returns
///
/// * `true` if the file exists, `false` otherwise.
///
/// # Example (in Lua)
///
/// ```lua
/// local deleted = fs.delete("/path/to/file")
/// print(deleted)
/// ```
pub(crate) fn delete_file(_lua: &Lua, (path, options): (String, Option<Table>)) -> mlua::Result<bool> {
    let path = PathBuf::from(&path);
    let mut recursive: bool = false;
    if let Some(ref tb) = options {
        if tb.contains_key("recursive")? {
            recursive = tb.get("recursive")?;
        }
    }

    match delete_recursively(path, recursive) {
        Ok(v) => Ok(v),
        Err(e) => Err(mlua::Error::RuntimeError(e.to_string())),
    }
}

pub(crate) fn is_dir(_lua: &Lua, path: String) -> mlua::Result<bool> {
    let path = PathBuf::from(&path);
    Ok(path.is_dir())
}

pub(crate) fn is_file(_lua: &Lua, path: String) -> mlua::Result<bool> {
    let path = PathBuf::from(&path);
    Ok(path.is_dir())
}

pub(crate) fn parent(_lua: &Lua, path: String) -> mlua::Result<Option<String>> {
    let path = PathBuf::from(&path).parent()
        .map(|x| x.to_str().unwrap().to_string());
    Ok(path)
}

fn delete_recursively<P: AsRef<Path>>(path: P, recursive: bool) -> io::Result<bool> {
    let path = path.as_ref();
    if path.is_dir() {
        if recursive {
            fs::remove_dir_all(path)?;
        } else {
            return Err(io::Error::new(ErrorKind::InvalidInput, "path is a directory"));
        }
    } else if path.is_file() {
        fs::remove_file(path)?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

pub(crate) fn read_file(lua: &Lua, path: String) -> mlua::Result<Value> {
    let s: Value = match fs::read_to_string(path) {
        Ok(s) => Value::String(lua.create_string(s.as_str())?),
        Err(_) => mlua::Value::Nil,
    };

    Ok(s)
}

pub(crate) fn write_file(_lua: &Lua, (path, value): (String, String)) -> mlua::Result<bool> {
    let res = fs::write(path, value);
    Ok(res.is_ok())
}