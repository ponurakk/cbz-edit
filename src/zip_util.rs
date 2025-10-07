use std::{
    fs,
    io::{Cursor, Read, Seek, Write},
    path::PathBuf,
};

use quick_xml::{
    Reader, Writer,
    de::from_str,
    events::{BytesText, Event},
    se::to_string,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::comic_info::ComicInfo;

/// Comment to add to `ComicInfo.xml`
const COMMENT: &str = " Modified by cbz-edit ";

/// Type alias for `ComicInfo` update callback
pub type ComicInfoUpdater = fn(old: ComicInfo, new: &ComicInfo) -> ComicInfo;

/// Modify a flat ZIP (no subdirectories) in-memory by replacing the file at `target_path` with
/// `new_comic_info`.
/// If `replace_all` is true, the contents of `new_comic_info` will be fully written to the file
/// else only the shared fields will be written.
fn modify_zip(
    input_path: &PathBuf,
    new_comic_info: &ComicInfo,
    updater: ComicInfoUpdater,
) -> anyhow::Result<()> {
    let input_zip = fs::read(input_path)?;
    let reader = Cursor::new(&input_zip);
    let mut zip = ZipArchive::new(reader)?;
    let mut out_buf = Vec::with_capacity(input_zip.len());

    {
        let cursor = Cursor::new(&mut out_buf);
        let mut writer = ZipWriter::new(cursor);
        let mut found_comic_info = false;

        for i in 0..zip.len() {
            let mut src = zip.by_index(i)?;
            let name = src.name().to_string();
            let opts = file_options(&src);

            if name == "ComicInfo.xml" {
                found_comic_info = true;
                write_comic_info(&mut writer, &mut src, new_comic_info, updater, &opts)?;
                continue;
            }

            copy_file(&mut writer, &mut src, &name, &opts)?;
        }

        if !found_comic_info {
            add_new_comic_info(&mut writer, new_comic_info, updater)?;
        }

        writer.finish()?;
    }

    fs::write(input_path, &out_buf)?;
    Ok(())
}

/// Get the options for a file
fn file_options<R>(src: &zip::read::ZipFile<R>) -> SimpleFileOptions
where
    R: Read,
{
    let mut opts = SimpleFileOptions::default().compression_method(src.compression());
    if let Some(mode) = src.unix_mode() {
        opts = opts.unix_permissions(mode);
    }
    opts
}

/// Updates `ComicInfo.xml` file from `src` and writes it to `writer`.
/// If `replace_all` is false, only the shared fields will be written
fn write_comic_info<W, R>(
    writer: &mut ZipWriter<W>,
    src: &mut zip::read::ZipFile<R>,
    new_info: &ComicInfo,
    updater: ComicInfoUpdater,
    opts: &SimpleFileOptions,
) -> anyhow::Result<()>
where
    W: Seek + Write,
    R: Read,
{
    writer.start_file("ComicInfo.xml", *opts)?;
    let mut content = String::new();
    src.read_to_string(&mut content)?;
    let old: ComicInfo = from_str(&content).unwrap_or_default();
    let updated_info = updater(old, new_info);
    let xml = to_string(&updated_info)?;

    let modified_xml = add_xml_comment(&xml, COMMENT)?;
    writer.write_all(modified_xml.as_bytes())?;
    Ok(())
}

/// Writes a new `ComicInfo.xml` file to `writer`.
/// If `replace_all` is false, only the shared fields will be written
fn add_new_comic_info<W>(
    writer: &mut ZipWriter<W>,
    new_info: &ComicInfo,
    updater: ComicInfoUpdater,
) -> anyhow::Result<()>
where
    W: Seek + Write,
{
    writer.start_file("ComicInfo.xml", SimpleFileOptions::default())?;
    let old = ComicInfo::default();
    let updated_info = updater(old, new_info);
    let xml = to_string(&updated_info)?;

    let modified_xml = add_xml_comment(&xml, COMMENT)?;
    writer.write_all(modified_xml.as_bytes())?;
    Ok(())
}

/// Copies a file from `src` to `writer`
fn copy_file<W, R>(
    writer: &mut ZipWriter<W>,
    src: &mut zip::read::ZipFile<R>,
    name: &str,
    opts: &SimpleFileOptions,
) -> anyhow::Result<()>
where
    W: Seek + Write,
    R: Read,
{
    let mut buf = Vec::with_capacity(usize::try_from(src.size())?);
    src.read_to_end(&mut buf)?;
    writer.start_file(name, *opts)?;
    writer.write_all(&buf)?;
    Ok(())
}

/// Inserts an XML comment at the top of an existing XML string using quick-xml.
fn add_xml_comment(xml: &str, comment: &str) -> anyhow::Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Comment(BytesText::new(comment)))?;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            e => writer.write_event(e)?,
        }
        buf.clear();
    }

    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

/// Overwrites everything
fn replace_all_updater(_old: ComicInfo, new: &ComicInfo) -> ComicInfo {
    new.clone()
}

/// Updates only shared fields
fn update_shared_updater(mut old: ComicInfo, new: &ComicInfo) -> ComicInfo {
    old.update_shared_fields(new);
    old
}

/// Updates fields derived from filename
fn derive_updater(mut old: ComicInfo, new: &ComicInfo) -> ComicInfo {
    old.volume = new.volume;
    old.number = new.number;
    old.translator.clone_from(&new.translator);
    old.title.clone_from(&new.title);
    old
}

/// Modify a flat ZIP (no subdirectories) in-memory by replacing the file at `target_path` with
/// `new_comic_info`.
pub fn modify_comic_info(path: &PathBuf, new_comic_info: &ComicInfo) -> anyhow::Result<()> {
    modify_zip(path, new_comic_info, update_shared_updater)
}

/// Replace the file at `target_path` with `new_comic_info`.
pub fn replace_comic_info(path: &PathBuf, new_comic_info: &ComicInfo) -> anyhow::Result<()> {
    modify_zip(path, new_comic_info, replace_all_updater)
}

/// Replace the file at `target_path` with `new_comic_info`.
pub fn derive_comic_info(path: &PathBuf, new_comic_info: &ComicInfo) -> anyhow::Result<()> {
    modify_zip(path, new_comic_info, derive_updater)
}

/// Get the `ComicInfo.xml` from a flat ZIP (no subdirectories)
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
