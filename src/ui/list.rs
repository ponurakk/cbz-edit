use std::{collections::HashSet, path::PathBuf};

use ratatui::widgets::{ListItem, ListState, ScrollbarState};
use tui_input::Input;

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

impl PartialEq for Series {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.name == other.name
    }
}
impl Eq for Series {}

impl PartialOrd for Series {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Series {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let c1 = &self.name.to_lowercase();
        let c2 = &other.name.to_lowercase();
        c1.partial_cmp(c2).unwrap_or(std::cmp::Ordering::Equal)
    }
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

    /// Search text
    pub search_text: Option<Input>,

    /// Found series from search
    pub found: (usize, Vec<usize>),
}

impl SeriesList {
    pub fn search(&mut self) {
        let Some(search_text) = &self.search_text else {
            return;
        };

        let filtered_indices: Vec<(Series, usize)> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.name
                    .to_lowercase()
                    .contains(&search_text.value().to_lowercase())
            })
            .map(|(idx, s)| (s.clone(), idx))
            .collect();

        if let Some(selected_idx) = filtered_indices.first() {
            self.state.select(Some(selected_idx.1));
        } else {
            self.state.select(None);
        }

        let mut found: Vec<usize> = filtered_indices.iter().map(|i| i.1).collect();
        found.sort_unstable();

        self.found = (0, found);
    }

    pub fn next_search(&mut self) {
        if let Some(input) = &self.search_text
            && input.value().is_empty()
        {
            return;
        }

        debug!("Search result: {:?}", self.found);
        if self.found.0 >= self.found.1.len().saturating_sub(1) {
            self.found.0 = 0;
        } else {
            self.found.0 += 1;
        }

        self.state.select(self.found.1.get(self.found.0).copied());
    }

    pub fn prev_search(&mut self) {
        if let Some(input) = &self.search_text
            && input.value().is_empty()
        {
            return;
        }

        debug!("Search result: {:?}", self.found);
        if self.found.0 == 0 {
            self.found.0 = self.found.1.len().saturating_sub(1);
        } else {
            self.found.0 = self.found.0.saturating_sub(1);
        }

        self.state.select(self.found.1.get(self.found.0).copied());
    }
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
            search_text: None,
            found: (0, Vec::new()),
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

impl PartialEq for Chapter {
    fn eq(&self, other: &Self) -> bool {
        self.volume == other.volume && self.chapter == other.chapter
    }
}
impl Eq for Chapter {}

impl PartialOrd for Chapter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Chapter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let c1 = self.chapter.unwrap_or(0.0);
        let c2 = other.chapter.unwrap_or(0.0);
        match c1.partial_cmp(&c2).unwrap_or(std::cmp::Ordering::Equal) {
            std::cmp::Ordering::Equal => self.path.cmp(&other.path),
            non_eq => non_eq,
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

    /// Custom field to track multiple selections
    pub selected: HashSet<usize>,
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
            selected: HashSet::new(),
        }
    }
}

impl ChapterList {
    pub fn toggle_selected(&mut self) {
        if let Some(index) = self.state.selected()
            && !self.selected.insert(index)
        {
            self.selected.remove(&index);
        }
    }
}

impl Chapter {
    pub fn get_title(&self, selected: bool) -> String {
        let selected_char = if selected {
            String::from("▌")
        } else {
            String::from(" ")
        };

        format!(
            "{}{:#5.}: {}",
            selected_char,
            self.chapter.unwrap_or_default(),
            self.title.clone().unwrap_or(
                self.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
        )
    }
}
