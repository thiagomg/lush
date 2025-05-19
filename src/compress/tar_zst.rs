use std::fs::File;
use std::{fs, io};
use std::path::PathBuf;

pub fn extract_tar_zst(path: PathBuf, output_dir: PathBuf) -> io::Result<()> {
    let file = File::open(path)?;
    let decoder = zstd::stream::Decoder::new(file)?;
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(output_dir)?;
    Ok(())
}

pub fn create_tar_zst(tar_zst_file: &PathBuf, src_files: &[PathBuf], recurse: bool) -> io::Result<()> {
    // Open output .tar.zst file
    let out_file = File::create(tar_zst_file)?;
    let encoder = zstd::Encoder::new(out_file, 0)?; // 0 = default compression level
    let mut tar_builder = tar::Builder::new(encoder);

    for path in src_files {
        if recurse {
            let metadata = fs::metadata(path)?;
            if metadata.is_file() {
                tar_builder.append_path_with_name(path, path.file_name().unwrap())?;
            } else if metadata.is_dir() {
                tar_builder.append_dir_all(path.file_name().unwrap(), path)?;
            }
        } else {
            let metadata = fs::metadata(path)?;
            if metadata.is_file() {
                tar_builder.append_path_with_name(path, path.file_name().unwrap())?;
            } else if metadata.is_dir() {
                tar_builder.append_dir(path.file_name().unwrap(), path)?;
            }
        }
    }

    // Finish both the tar and zstd writers
    tar_builder.into_inner()?.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_extract_tar_zst() {
        // Setup: create a temp dir with a file
        let temp_dir = tempdir().unwrap();
        let src_file_path = temp_dir.path().join("example.txt");
        let mut file = File::create(&src_file_path).unwrap();
        writeln!(file, "Hello from the tar.zst test!").unwrap();

        // Path to .tar.zst file
        let archive_path = temp_dir.path().join("archive.tar.zst");

        // Create .tar.zst
        create_tar_zst(&archive_path, &[src_file_path.clone()], false).unwrap();
        assert!(archive_path.exists());

        // Extract it to a different temp dir
        let extract_dir = tempdir().unwrap();
        extract_tar_zst(archive_path.clone(), extract_dir.path().to_path_buf()).unwrap();

        // Check that the file exists and content is correct
        let extracted_file_path = extract_dir.path().join("example.txt");
        assert!(extracted_file_path.exists());

        let content = fs::read_to_string(extracted_file_path).unwrap();
        assert_eq!(content.trim(), "Hello from the tar.zst test!");
    }
}
