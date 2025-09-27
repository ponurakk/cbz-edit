//! Data management

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::ui::list::{Chapter, ChapterList, Series};

pub fn get_series_list<P: AsRef<Path>>(path: P) -> io::Result<Vec<Series>> {
    let mut folders = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            let mut chapters = crate::data::get_cbz_list(entry.path())?;
            chapters.sort_by(|a, b| {
                a.chapter
                    .unwrap_or_default()
                    .partial_cmp(&b.chapter.unwrap_or_default())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            folders.push(Series {
                name: entry.file_name().into_string().unwrap_or_default(),
                path: entry.path(),
                chapters: ChapterList::from_iter(chapters),
            });
        }
    }

    Ok(folders)
}

pub fn get_cbz_list<P: AsRef<Path>>(path: P) -> io::Result<Vec<Chapter>> {
    let mut cbz_files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && let Some(ext) = path.extension()
            && ext.eq_ignore_ascii_case("cbz")
            && let Some(name) = path.file_name()
        {
            cbz_files.push(
                parse_filename(path.clone(), name.to_string_lossy().as_ref()).unwrap_or_default(),
            );
        }
    }

    Ok(cbz_files)
}

// TODO: Improve this or use something better than regex
fn parse_filename(path: PathBuf, filename: &str) -> Option<Chapter> {
    let re = Regex::new(
        r"(?x)
(?:Vol\.?\s*(?P<vol>\d+))? # Volume
[\s.-]*
(?:Ch(?:ap(?:ter)?)?\.?\s*(?P<ch>\d+(?:\.\d+)?))? # Chapter
[\s:.-]*
(?P<title>.+?)                 # capture as much as possible
(?:\\s*\\((?P<lang>[a-z]{2,3})\\))?  # only treat the last (...) with 2-3 letters as language
(?:\\s*\\[(?P<translators>[^]]+)\\])? # translators
\.cbz$
",
    )
    .ok()?;

    let caps = re.captures(filename)?;

    let volume = caps
        .name("vol")
        .and_then(|v| v.as_str().parse::<u32>().ok());
    let chapter = caps.name("ch").and_then(|c| c.as_str().parse::<f32>().ok());
    let title = caps
        .name("title")
        .map(|t| {
            t.as_str()
                .trim_matches(|c: char| c == '-' || c == ':' || c.is_whitespace())
                .to_string()
        })
        .filter(|s| !s.is_empty());

    let translators = caps
        .name("translators")
        .map(|t| {
            t.as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(Chapter {
        path,
        volume,
        chapter,
        title,
        translators,
    })
}
