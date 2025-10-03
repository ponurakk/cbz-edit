use std::path::PathBuf;

use ratatui::widgets::{ListItem, ListState, ScrollbarState};

/// Series from disk
#[derive(Debug, Clone, Default)]
pub struct Series {
    /// Path to the series
    pub path: PathBuf,

    /// Name of the series
    pub name: String,

    /// Chapters of the series
    pub chapters: ChapterList,
}

/// List of series
pub struct SeriesList {
    /// Shown state of projects
    pub items: Vec<Series>,

    /// Initial static state of projects
    pub items_state: Vec<Series>,

    /// State of the list
    pub state: ListState,

    /// State of the scrollbar
    pub scroll_state: ScrollbarState,
}

impl FromIterator<Series> for SeriesList {
    fn from_iter<T: IntoIterator<Item = Series>>(iter: T) -> Self {
        let mut state = ListState::default();
        state.select_first();

        let items: Vec<Series> = iter.into_iter().collect();
        let len = items.len();
        Self {
            items: items.clone(),
            items_state: items,
            state,
            scroll_state: ScrollbarState::default().content_length(len),
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

/// List of chapters in a series
#[derive(Debug, Clone, Default)]
pub struct ChapterList {
    /// Shown state of projects
    pub items: Vec<Chapter>,

    /// Initial static state of projects
    pub items_state: Vec<Chapter>,

    /// State of the list
    pub state: ListState,

    /// State of the scrollbar
    pub scroll_state: ScrollbarState,
}

impl FromIterator<Chapter> for ChapterList {
    fn from_iter<T: IntoIterator<Item = Chapter>>(iter: T) -> Self {
        let mut state = ListState::default();
        state.select_first();

        let items: Vec<Chapter> = iter.into_iter().collect();
        let len = items.len();
        Self {
            items: items.clone(),
            items_state: items,
            state,
            scroll_state: ScrollbarState::default().content_length(len),
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
