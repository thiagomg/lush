use std::fs::{File, metadata};
use std::{fs, io};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use mlua::{Lua, Value, Variadic};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use zip::result::ZipResult;

pub(crate) fn zip_deflate(_lua: &Lua, (zip_name, files_to_add): (String, Variadic<Value>)) -> mlua::Result<()> {
    let mut files = vec![];
    for (_i, arg) in files_to_add.iter().enumerate() {
        files.push(PathBuf::from(arg.to_string()?));
    }
    zip_deflate_int(&PathBuf::from(&zip_name), &files, true)?;
    Ok(())
}

fn zip_deflate_int(zip_file: &PathBuf, src_files: &[PathBuf], recurse: bool) -> io::Result<()> {
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

pub(crate) fn zip_inflate(_lua: &Lua, (zip_name, output_dir): (String, Option<String>)) -> mlua::Result<()> {
    let output_dir = output_dir.unwrap_or(".".to_string());
    if let Err(e) = zip_inflate_int(PathBuf::from(zip_name), PathBuf::from(output_dir)) {
        return Err(io::Error::new(ErrorKind::InvalidData, e.to_string()).into());
    }
    Ok(())
}

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
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
