use std::fs::{File, metadata};
use std::{fs, io};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use mlua::{Lua, Value, Variadic};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use zip::result::ZipResult;

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
pub(crate) fn zip_deflate(_lua: &Lua, (zip_name, files_to_add): (String, Variadic<Value>)) -> mlua::Result<()> {
    let mut files = vec![];
    for arg in files_to_add.iter() {
        files.push(PathBuf::from(arg.to_string()?));
    }
    zip_deflate_int(&PathBuf::from(&zip_name), &files, true)?;
    Ok(())
}

/// Internal function to handle the zipping process.
///
/// This function creates a ZIP file and writes the specified source files into it. It can handle both files and directories,
/// zipping them recursively if specified.
///
/// # Arguments
///
/// * `zip_file` - The path to the ZIP file to create.
/// * `src_files` - A slice of paths to source files to add to the ZIP archive.
/// * `recurse` - A boolean indicating whether to zip directories recursively.
///
/// # Returns
///
/// * An `io::Result` indicating success or an error if the zipping fails.
fn zip_deflate_int(zip_file: &PathBuf, src_files: &[PathBuf], recurse: bool) -> io::Result<()> {
    let dest_file = File::create(zip_file)?;
    let mut writer = zip::ZipWriter::new(dest_file);
    let mut buffer = Vec::new();

    zip_list(src_files, &mut writer, &mut buffer, recurse)?;

    writer.finish()?;
    Ok(())
}

/// Writes the contents of the specified source files to the ZIP writer.
///
/// This function handles both files and directories, adding them to the ZIP archive while preserving their structure.
///
/// # Arguments
///
/// * `src_files` - A slice of paths to source files or directories to add to the ZIP archive.
/// * `writer` - A mutable reference to the `ZipWriter` to write the files into.
/// * `buffer` - A mutable reference to a buffer used for reading file contents.
/// * `recurse` - A boolean indicating whether to add files in directories recursively.
///
/// # Returns
///
/// * An `io::Result` indicating success or an error if the operation fails.
fn zip_list(src_files: &[PathBuf], writer: &mut ZipWriter<File>, buffer: &mut Vec<u8>, recurse: bool) -> io::Result<()> {
    for src_path in src_files.iter() {
        let md = metadata(src_path).unwrap();
        if md.is_file() {
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(md.permissions().mode());

            writer.start_file(src_path.to_str().unwrap(), options)?;
            let mut f = File::open(src_path)?;
            f.read_to_end(buffer)?;
            writer.write_all(buffer)?;
            buffer.clear();
        } else if md.is_dir() {
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            writer.add_directory(src_path.to_str().unwrap(), options)?;
            if recurse {
                let files: Vec<PathBuf> = fs::read_dir(src_path)?
                    .map(|f| f.unwrap().path()).collect();

                zip_list(&files, writer, buffer, recurse)?;
            }
        }
    }
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
pub(crate) fn zip_inflate(_lua: &Lua, (zip_name, output_dir): (String, Option<String>)) -> mlua::Result<()> {
    let output_dir = output_dir.unwrap_or(".".to_string());
    if let Err(e) = zip_inflate_int(PathBuf::from(zip_name), PathBuf::from(output_dir)) {
        return Err(io::Error::new(ErrorKind::InvalidData, e.to_string()).into());
    }
    Ok(())
}

/// Internal function to handle the extraction of ZIP archives.
///
/// This function reads the contents of a ZIP file and extracts them to the specified output directory.
///
/// # Arguments
///
/// * `path` - The path to the ZIP file to extract.
/// * `output_dir` - The directory where the contents will be extracted.
///
/// # Returns
///
/// * A `ZipResult` indicating success or an error if the extraction fails.
fn zip_inflate_int(path: PathBuf, output_dir: PathBuf) -> ZipResult<()> {
    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = output_dir.join(file.name());

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
