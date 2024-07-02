use std::fs::{File, metadata};
use std::{fs, io};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn zip_deflate(zip_file: &PathBuf, src_files: &[PathBuf], recurse: bool) -> io::Result<()> {
    let dest_file = File::create(zip_file)?;
    let mut writer = zip::ZipWriter::new(dest_file);
    let mut buffer = Vec::new();

    zip_list(src_files, &mut writer, &mut buffer, recurse)?;

    writer.finish()?;
    Ok(())
}

fn zip_list(src_files: &[PathBuf], writer: &mut ZipWriter<File>, mut buffer: &mut Vec<u8>, recurse: bool) -> io::Result<()> {
    for src_path in src_files.iter() {
        let md = metadata(src_path).unwrap();
        if md.is_file() {
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(md.permissions().mode());

            writer.start_file(src_path.to_str().unwrap(), options)?;
            let mut f = File::open(src_path)?;
            f.read_to_end(&mut buffer)?;
            writer.write_all(&buffer)?;
            buffer.clear();
        } else if md.is_dir() {
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            writer.add_directory(src_path.to_str().unwrap(), options)?;
            if recurse {
                let files: Vec<PathBuf> = fs::read_dir(&src_path)?
                    .into_iter().map(|f| f.unwrap().path()).collect();

                zip_list(&files, writer, buffer, recurse)?;
            }
        }
    }
    Ok(())
}