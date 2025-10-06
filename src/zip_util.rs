use std::{
    fs,
    io::{Cursor, Read, Write},
    path::PathBuf,
};

use quick_xml::de::from_str;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::comic_info::ComicInfo;

/// Modify a flat ZIP (no subdirectories) in-memory by replacing the file at `target_path` with
/// `new_content`. Returns a new ZIP as Vec<u8>
pub fn modify_zip_in_memory(input_path: &PathBuf, new_content: &[u8]) -> anyhow::Result<()> {
    let input_zip = fs::read(&input_path)?;
    let reader = Cursor::new(&input_zip);
    let mut zip = ZipArchive::new(reader)?;

    let mut out_buf: Vec<u8> = Vec::with_capacity(input_zip.len());
    {
        let cursor = Cursor::new(&mut out_buf);
        let mut writer = ZipWriter::new(cursor);

        let mut found_comic_info = false;

        for i in 0..zip.len() {
            let mut src_file = zip.by_index(i)?;
            let name = src_file.name().to_string();

            let mut options =
                SimpleFileOptions::default().compression_method(src_file.compression());

            if let Some(mode) = src_file.unix_mode() {
                options = options.unix_permissions(mode);
            }

            if name == "ComicInfo.xml" {
                found_comic_info = true;
                writer.start_file("ComicInfo.xml", options)?;
                writer.write_all(new_content)?;
            } else {
                let mut buf = Vec::with_capacity(src_file.size() as usize);
                src_file.read_to_end(&mut buf)?;
                writer.start_file(name, options)?;
                writer.write_all(&buf)?;
            }
        }

        // Add ComicInfo.xml if it wasn’t in the archive
        if !found_comic_info {
            writer.start_file("ComicInfo.xml", SimpleFileOptions::default())?;
            writer.write_all(new_content)?;
        }

        writer.finish()?;
    }

    drop(zip);
    drop(input_zip);

    fs::write(&input_path, &out_buf)?;

    drop(out_buf);

    Ok(())
}

pub fn get_comic_from_zip(path: &PathBuf) -> anyhow::Result<ComicInfo> {
    let input_zip = fs::read(path)?;
    let reader = Cursor::new(input_zip);
    let mut archive = ZipArchive::new(reader)?;

    match archive.by_name("ComicInfo.xml") {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(from_str(&content).unwrap_or_default())
        }
        Err(_) => Ok(ComicInfo::default()), // file not found
    }
}
