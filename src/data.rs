//! Data management

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::ui::list::{Chapter, ChapterList, Series};

pub fn get_series_list<P: AsRef<Path>>(path: P) -> io::Result<Vec<Series>> {
    let mut folders = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            let mut chapters = crate::data::get_cbz_list(entry.path())?;
            chapters.sort();
            folders.push(Series {
                name: entry.file_name().into_string().unwrap_or_default(),
                path: entry.path(),
                chapters: ChapterList::from_iter(chapters),
            });
        }
    }

    folders.sort();

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
            cbz_files.push(parse_filename(
                path.clone(),
                name.to_string_lossy().as_ref(),
            ));
        }
    }

    Ok(cbz_files)
}

pub fn parse_filename(path: PathBuf, filename: &str) -> Chapter {
    let name = filename.trim_end_matches(".cbz");

    // --- translators ---
    let mut translators = Vec::new();
    let mut core = name.to_string();
    if let Some(start) = core.rfind('[')
        && let Some(end) = core.rfind(']')
    {
        let inside = &core[start + 1..end];
        translators = inside.split(',').map(|s| s.trim().to_string()).collect();
        core = core[..start].trim().to_string();
    }

    // --- language ---
    let mut lang: Option<String> = None;
    if let Some(start) = core.rfind('(')
        && let Some(end) = core.rfind(')')
    {
        let inside = &core[start + 1..end];
        if inside.len() >= 2 && inside.len() <= 3 {
            lang = Some(inside.to_string());
            core = core[..start].trim().to_string();
        }
    }

    // --- normalize separators ---
    let separators = |c: char| c.is_whitespace() || c == '-' || c == ':';
    let tokens: Vec<&str> = core.split(separators).filter(|s| !s.is_empty()).collect();

    let mut volume: Option<u32> = None;
    let mut chapter: Option<f32> = None;
    let mut leftovers = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let tok = tokens[i];
        let low = tok.to_lowercase();

        if low.starts_with("vol") {
            // handle "Vol.02" or "Vol02"
            if let Ok(num_str) = tok
                .trim_start_matches(|c: char| !c.is_ascii_digit())
                .parse::<u32>()
            {
                volume = Some(num_str);
            } else if let Some(next) = tokens.get(i + 1)
                && let Ok(v) = next.parse::<u32>()
            {
                volume = Some(v);
                i += 1; // skip number token
            }
        } else if low.starts_with("ch")
            || low.starts_with("chap")
            || low.starts_with("chapter")
            || low.starts_with("ep")
            || low.starts_with("episode")
            || low.starts_with("#")
        {
            // Remove prefix letters like "Ch", "Chap.", etc.
            let mut num_str =
                tok.trim_start_matches(|c: char| c.is_alphabetic() || c == '.' || c == '#');

            // Sometimes chapter number is in next token
            if num_str.is_empty()
                && let Some(next) = tokens.get(i + 1)
            {
                num_str = next;
                i += 1;
            }

            if let Ok(c) = num_str.parse::<f32>() {
                chapter = Some(c);
            }
        } else if let Ok(num) = tok.parse::<f32>() {
            if chapter.is_none() {
                chapter = Some(num);
            } else {
                leftovers.push(tok.to_string());
            }
        } else {
            leftovers.push(tok.to_string());
        }

        i += 1; // always advance!
    }

    let title = if leftovers.is_empty() {
        None
    } else {
        Some(leftovers.join(" "))
    };

    Chapter {
        path,
        volume,
        chapter,
        title,
        translators,
    }
}
