use std::path::PathBuf;

use ratatui::widgets::{ListItem, ListState};

#[derive(Debug, Clone, Default)]
pub struct Series {
    pub path: PathBuf,
    pub name: String,
    pub chapters: ChapterList,
}

pub struct SeriesList {
    /// Shown state of projects
    pub items: Vec<Series>,

    /// Initial static state of projects
    pub items_state: Vec<Series>,

    /// State of the list
    pub state: ListState,
}

impl FromIterator<Series> for SeriesList {
    fn from_iter<T: IntoIterator<Item = Series>>(iter: T) -> Self {
        let state = ListState::default();
        let items: Vec<Series> = iter.into_iter().collect();
        Self {
            items: items.clone(),
            items_state: items,
            state,
        }
    }
}

impl From<&Series> for ListItem<'_> {
    fn from(value: &Series) -> Self {
        ListItem::new(value.name.clone())
    }
}

/// Chapter of a series from disk
#[derive(Debug, Clone)]
pub struct Chapter {
    /// Path to the chapter (cbz file)
    pub path: PathBuf,

    /// Volume of the chapter
    pub volume: Option<u32>,

    /// Chapter number
    #[allow(clippy::struct_field_names)]
    pub chapter: Option<f32>,

    /// Title of the chapter
    pub title: Option<String>,

    /// Translators
    pub translators: Vec<String>,
}

impl Default for Chapter {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            volume: None,
            chapter: None,
            title: None,
            translators: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChapterList {
    /// Shown state of projects
    pub items: Vec<Chapter>,

    /// Initial static state of projects
    pub items_state: Vec<Chapter>,

    /// State of the list
    pub state: ListState,
}

impl FromIterator<Chapter> for ChapterList {
    fn from_iter<T: IntoIterator<Item = Chapter>>(iter: T) -> Self {
        let state = ListState::default();
        let items: Vec<Chapter> = iter.into_iter().collect();
        Self {
            items: items.clone(),
            items_state: items,
            state,
        }
    }
}

impl From<&Chapter> for ListItem<'_> {
    fn from(value: &Chapter) -> Self {
        ListItem::new(format!(
            "{:#5.}: {}",
            value.chapter.unwrap_or_default(),
            value.title.clone().unwrap_or(
                value
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
        ))
    }
}
