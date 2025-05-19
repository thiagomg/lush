use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;
use mlua::{Lua, Value, Variadic};
use crate::compress::tar_zst::{create_tar_zst, extract_tar_zst};
use crate::compress::zip::{create_zip_int, extract_zip_int};

/// Compresses the specified files into a compressed archive.
///
/// This function takes the name of the compressed file to create and a variadic list of file paths to add to the archive.
/// It calls a helper function to handle the actual zipping process.
/// Currently, only zip and tar.zst are supported
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `(dest_file_name, files_to_add)` - A tuple where:
///   - `dest_file_name` (String): The name of the resulting compressed file.
///   - `files_to_add` (`Variadic<Value>`): A variadic list of files to include in the ZIP archive.
///
/// # Returns
///
/// * A `Result` indicating success or an error if the zipping fails.
///
/// # Example
///
/// ```lua
/// files.compress("archive.zip", "file1.txt", "file2.txt")
/// files.compress("archive.tar.zst", "file1.txt", "dir1")
/// ```
pub(crate) fn compress(_lua: &Lua, (dest_file_name, files_to_add): (String, Variadic<Value>)) -> mlua::Result<()> {
    let mut files = vec![];
    for arg in files_to_add.iter() {
        files.push(PathBuf::from(arg.to_string()?));
    }

    let dest_file_path = PathBuf::from(&dest_file_name);
    if dest_file_name.ends_with(".zip") {
        create_zip_int(&dest_file_path, &files, true)?;
    } else if dest_file_name.ends_with(".tar.zst") {
        create_tar_zst(&dest_file_path, &files, true)?;
    } else {
        return Err(mlua::Error::RuntimeError("Unsupported compression algorithm. Valid extensions are: .zip and .tar.zst".to_string()));
    }
    Ok(())
}

/// Decompresses a compressed archive into the specified output directory.
///
/// This function takes the name of the compressed file and an optional output directory. If no output directory is provided,
/// the current directory is used as the default destination.
/// Currently, only zip and tar.zst are supported
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `(src_file_name, output_dir)` - A tuple where:
///   - `src_file_name` (String): The name of the compressed file to extract.
///   - `output_dir` (`Option<String>`): An optional path to the directory where the contents will be extracted.
///
/// # Returns
///
/// * A `Result` indicating success or an error if the extraction fails.
///
/// # Example
///
/// ```lua
/// files.decompress("archive.zip", "output_directory")
/// files.decompress("archive.tar.zst", "output_directory")
/// ```
pub(crate) fn decompress(_lua: &Lua, (src_file_name, output_dir): (String, Option<String>)) -> mlua::Result<()> {
    let output_dir = output_dir.unwrap_or(".".to_string());
    let src_file_path = PathBuf::from(&src_file_name);
    let output_dir_path = PathBuf::from(output_dir);

    if src_file_name.ends_with(".zip") {
        if let Err(e) = extract_zip_int(src_file_path, output_dir_path) {
            return Err(io::Error::new(ErrorKind::InvalidData, e.to_string()).into());
        }
    } else if src_file_name.ends_with(".tar.zst") {
        extract_tar_zst(src_file_path, output_dir_path)?;
    } else {
        return Err(mlua::Error::RuntimeError("Unsupported compression algorithm. Valid extensions are: .zip and .tar.zst".to_string()));
    }

    Ok(())
}

/// Compresses the specified files into a ZIP archive.
///
/// This function takes the name of the ZIP file to create and a variadic list of file paths to add to the archive.
/// It calls a helper function to handle the actual zipping process.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `(zip_name, files_to_add)` - A tuple where:
///   - `zip_name` (String): The name of the resulting ZIP file.
///   - `files_to_add` (`Variadic<Value>`): A variadic list of files to include in the ZIP archive.
///
/// # Returns
///
/// * A `Result` indicating success or an error if the zipping fails.
///
/// # Example
///
/// ```lua
/// files.zip("archive.zip", "file1.txt", "file2.txt")
/// ```
pub(crate) fn create_zip(_lua: &Lua, (zip_name, files_to_add): (String, Variadic<Value>)) -> mlua::Result<()> {
    let mut files = vec![];
    for arg in files_to_add.iter() {
        files.push(PathBuf::from(arg.to_string()?));
    }
    create_zip_int(&PathBuf::from(&zip_name), &files, true)?;
    Ok(())
}

/// Decompresses a ZIP archive into the specified output directory.
///
/// This function takes the name of the ZIP file and an optional output directory. If no output directory is provided,
/// the current directory is used as the default destination.
///
/// # Arguments
///
/// * `_lua` - The Lua state (not used in this function).
/// * `(zip_name, output_dir)` - A tuple where:
///   - `zip_name` (String): The name of the ZIP file to extract.
///   - `output_dir` (`Option<String>`): An optional path to the directory where the contents will be extracted.
///
/// # Returns
///
/// * A `Result` indicating success or an error if the extraction fails.
///
/// # Example
///
/// ```lua
/// files.unzip("archive.zip", "output_directory")
/// ```
pub(crate) fn extract_zip(_lua: &Lua, (zip_name, output_dir): (String, Option<String>)) -> mlua::Result<()> {
    let output_dir = output_dir.unwrap_or(".".to_string());
    if let Err(e) = extract_zip_int(PathBuf::from(zip_name), PathBuf::from(output_dir)) {
        return Err(io::Error::new(ErrorKind::InvalidData, e.to_string()).into());
    }
    Ok(())
}
