use std::fs::{File, metadata};
use std::{fs, io};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{Local, NaiveDateTime, TimeZone};
use filetime::{set_file_mtime, FileTime};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use zip::result::ZipResult;

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
pub(crate) fn create_zip_int(zip_file: &PathBuf, src_files: &[PathBuf], recurse: bool) -> io::Result<()> {
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
pub(crate) fn extract_zip_int(path: PathBuf, output_dir: PathBuf) -> ZipResult<()> {
    let zip_file = File::open(&path)?;
    let mut archive = ZipArchive::new(zip_file)?;

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
            drop(outfile);
        }

        // Restore permissions (if available)
        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            set_permissions(&out_path, mode)?;
        }

        if let Some(last_mod) = file.last_modified() {
            if let Ok(dt) = NaiveDateTime::try_from(last_mod) {
                let _ = set_mtime_from_local_naive(&out_path, dt);
            }

        }
    }


    Ok(())
}

fn set_mtime_from_local_naive(path: &Path, naive: NaiveDateTime) -> std::io::Result<()> {
    // Interpret NaiveDateTime as local time
    let local_dt = Local.from_local_datetime(&naive)
        .single()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Ambiguous or nonexistent local time"))?;

    // Convert to SystemTime
    let system_time: SystemTime = local_dt.into();

    // Convert to FileTime and set mtime
    let file_time = FileTime::from_system_time(system_time);
    set_file_mtime(path, file_time)
}

#[cfg(unix)]
fn set_permissions(path: &PathBuf, mode: u32) -> io::Result<()> {
    use std::fs::Permissions;
    fs::set_permissions(path, Permissions::from_mode(mode))
}
