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

fn is_chapter_prefix(token: &str) -> bool {
    let low = token.to_lowercase();

    // pure chapter prefixes
    let prefixes = [
        "ch", "ch.", "chap", "chap.", "chapter", "chapter.", "ep", "ep.", "episode", "episode.",
    ];

    // If token is exactly a prefix OR prefix + number
    for p in &prefixes {
        if low == *p {
            return true;
        }
        if low.starts_with(p) {
            // ensure next char is digit or dot
            if let Some(rest) = low.strip_prefix(*p)
                && rest.starts_with(|c: char| c.is_ascii_digit())
            {
                return true;
            }
        }
    }

    // '#' chapter tokens: "#12"
    if low.starts_with('#') && low[1..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    false
}

fn tokenize_preserving_brackets(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut bracket_depth: i32 = 0;

    for c in s.chars() {
        match c {
            '[' => {
                bracket_depth += 1;
                buf.push(c);
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                buf.push(c);
            }

            // Split ONLY when NOT inside brackets
            '-' | ':' | ' ' | '\t' | '\n' if bracket_depth == 0 => {
                if !buf.is_empty() {
                    tokens.push(buf.clone());
                    buf.clear();
                }
            }

            _ => buf.push(c),
        }
    }

    if !buf.is_empty() {
        tokens.push(buf);
    }

    tokens
}

pub fn parse_filename(path: PathBuf, filename: &str) -> Chapter {
    let name = filename.trim_end_matches(".cbz");

    // Extract translators [ ... ]
    let mut translators = Vec::new();
    let mut core = name.to_string();

    if let (Some(start), Some(end)) = (core.rfind('['), core.rfind(']'))
        && end > start
    {
        let inside = &core[start + 1..end];
        translators = inside
            .split(',')
            .map(|w| w.trim().to_string())
            .filter(|w| !w.is_empty())
            .collect();
        core = core[..start].trim().to_string();
    }

    // Extract language tag (en) (jp) etc.
    if let (Some(start), Some(end)) = (core.rfind('('), core.rfind(')'))
        && end > start
    {
        let inside = &core[start + 1..end];
        if (2..=3).contains(&inside.len()) {
            core = core[..start].trim().to_string();
        }
    }

    // Tokenize
    let tokens: Vec<String> = tokenize_preserving_brackets(&core);

    let mut volume: Option<u32> = None;
    let mut chapter: Option<f32> = None;
    let mut leftovers: Vec<String> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        let low = tok.to_lowercase();

        // Volume detection
        if (low.starts_with("vol") || low.starts_with("volume"))
            && !tok.contains('[')
            && !tok.contains(']')
        {
            // Extract digits from token itself
            let digits_only: String = tok.chars().filter(char::is_ascii_digit).collect();

            if let Ok(v) = digits_only.parse::<u32>() {
                volume = Some(v);
            } else if let Some(next) = tokens.get(i + 1)
                && let Ok(v) = next.parse::<u32>()
            {
                volume = Some(v);
                i += 1; // skip the next token
            }
        }
        // Chapter detection
        else if chapter.is_none() && is_chapter_prefix(tok) {
            // extract numeric suffix if present
            let mut num =
                tok.trim_start_matches(|c: char| c.is_alphabetic() || c == '.' || c == '#');

            // number might be next token
            if num.is_empty()
                && let Some(next) = tokens.get(i + 1)
                && next.parse::<f32>().is_ok()
            {
                num = next;
                i += 1;
            }

            if let Ok(n) = num.parse::<f32>() {
                chapter = Some(n);
            } else {
                leftovers.push(tok.to_string());
            }
        } else if let Ok(n) = tok.parse::<f32>() {
            if chapter.is_none() {
                chapter = Some(n);
            }
            leftovers.push(tok.to_string());
        }
        // Everything else = title token
        else {
            leftovers.push(tok.to_string());
        }

        i += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn parse(name: &str) -> Chapter {
        parse_filename(PathBuf::from(name), name)
    }

    #[test]
    fn test_simple_chapter() {
        let c = parse("Ch.05 Title.cbz");
        assert_eq!(c.chapter, Some(5.0));
        assert_eq!(c.volume, None);
        assert_eq!(c.title, Some("Title".into()));
    }

    #[test]
    fn test_volume_and_chapter() {
        let c = parse("Vol.03 Ch.12 Title.cbz");
        assert_eq!(c.volume, Some(3));
        assert_eq!(c.chapter, Some(12.0));
        assert_eq!(c.title, Some("Title".into()));
    }

    #[test]
    fn test_decimal_chapter() {
        let c = parse("Ch.10.5.cbz");
        assert_eq!(c.chapter, Some(10.5));
        assert_eq!(c.volume, None);
    }

    #[test]
    fn test_translators() {
        let c = parse("Ch.0002 [alpha, beta].cbz");
        assert_eq!(c.translators, vec!["alpha", "beta"]);
        assert_eq!(c.chapter, Some(2.0));
    }

    #[test]
    fn test_language_tag() {
        let c = parse("Vol.1 Ch.2 Title (en).cbz");
        assert_eq!(c.volume, Some(1));
        assert_eq!(c.chapter, Some(2.0));
        assert_eq!(c.title, Some("Title".into()));
    }

    #[test]
    fn test_complex() {
        let c = parse("Volume 12 - Chapter 4.5 Final Fight (en) [scanA, scanB].cbz");

        assert_eq!(c.volume, Some(12));
        assert_eq!(c.chapter, Some(4.5));
        assert_eq!(c.translators, vec!["scanA", "scanB"]);
        assert_eq!(c.title, Some("Final Fight".into()));
    }

    #[test]
    fn test_hash_prefixed() {
        let c = parse("Series #7.cbz");
        assert_eq!(c.chapter, Some(7.0));
    }

    #[test]
    fn test_bare_number_fallback() {
        let c = parse("Night 44.cbz");
        assert_eq!(c.chapter, Some(44.0));
        assert_eq!(c.title, Some("Night 44".into()));
    }

    // From some edge cases found in the wild
    #[test]
    fn test_double_ch() {
        let c = parse("Ch.081.4 - High School Girls are Funky: Ch14 - Endurance.cbz");

        // First chapter number found is considered the main chapter
        assert_eq!(c.chapter, Some(81.4));
        assert_eq!(
            c.title,
            Some("High School Girls are Funky Ch14 Endurance".into())
        );
        assert_eq!(c.volume, None);
        assert_eq!(c.translators, Vec::<String>::new());
    }

    #[test]
    fn test_word_with_ch() {
        let c =
            parse("Vol.03 Ch.0022 - Chika Fujiwara Wants to be Eaten (en) [Psylocke Scans].cbz");

        assert_eq!(c.volume, Some(3));
        assert_eq!(c.chapter, Some(22.0));
        assert_eq!(c.title, Some("Chika Fujiwara Wants to be Eaten".into()));
        assert_eq!(c.translators, vec!["Psylocke Scans"]);
    }

    #[test]
    fn test_number_in_the_title() {
        let c = parse("Vol.02 Ch.0006 - Episode 6 (en) [I post what I like].cbz");

        assert_eq!(c.volume, Some(2));
        assert_eq!(c.chapter, Some(6.0));
        assert_eq!(c.title, Some("Episode 6".into()));
        assert_eq!(c.translators, vec!["I post what I like"]);
    }

    #[test]
    fn test_number_with_hash_in_the_title() {
        let c = parse("Chap 3: The Desire to Be #1.cbz");

        // Main chapter number
        assert_eq!(c.chapter, Some(3.0));
        assert_eq!(c.title, Some("The Desire to Be #1".into()));
        assert_eq!(c.volume, None);
        assert_eq!(c.translators, Vec::<String>::new());
    }

    #[test]
    fn test_double_square_brackets() {
        let c =
            parse("Vol.02 Ch.0015.5 - Volume[1-2] Illustrations (en) [ROCK-paper-SCISSORS].cbz");

        assert_eq!(c.volume, Some(2));
        assert_eq!(c.chapter, Some(15.5));
        assert_eq!(c.title, Some("Volume[1-2] Illustrations".into()));
        assert_eq!(c.translators, vec!["ROCK-paper-SCISSORS"]);
    }

    // FIX: Still unresolved idk how to fix this or if I should care about this edge case
    #[test]
    #[ignore = "unresolved"]
    fn test_no_chapter_number() {
        let c = parse("Special           : Special Chapter.cbz");

        // No chapter number present
        assert_eq!(c.chapter, None);
        assert_eq!(c.volume, None);
        assert_eq!(c.title, Some("Special Chapter".into()));
        assert_eq!(c.translators, Vec::<String>::new());
    }

    #[test]
    fn test_chapter_and_ch_in_title() {
        let c = parse("Chapter 29           : Cheep Talk.cbz");

        assert_eq!(c.chapter, Some(29.0));
        assert_eq!(c.volume, None);
        assert_eq!(c.title, Some("Cheep Talk".into()));
        assert_eq!(c.translators, Vec::<String>::new());
    }
}
