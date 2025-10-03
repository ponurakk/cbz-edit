//! UI for the application

use std::path::PathBuf;

use image::ImageReader;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

use crate::{
    comic_info::ComicInfo,
    ui::list::{Chapter, Series, SeriesList},
    zip_util::get_comic_from_zip,
};

pub mod app;
pub mod list;

/// Current tab
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    SeriesList,
    ChaptersList,
    Metadata,
}

/// Current comic selected on chapter list
pub struct ComicState {
    info: ComicInfo,
    fields: Vec<&'static str>,
    field_inputs: Vec<String>,
    fields_state: ListState,
}

/// Main application
pub struct App {
    should_exit: bool,
    current_tab: Tab,
    series_list: SeriesList,
    image: StatefulProtocol,

    comic: ComicState,
}

impl Default for App {
    fn default() -> Self {
        Self::new(vec![]).expect("Failed to create app")
    }
}

impl App {
    /// Create a new application
    pub fn new(series_list: Vec<Series>) -> anyhow::Result<Self> {
        let dyn_img =
            ImageReader::open("tumblr_586a38213908da1a27f7d49cf4fed52b_ba0d374c_1280.jpg")?
                .decode()?;
        let picker = Picker::from_fontsize((8, 16));
        let protocol = picker.new_resize_protocol(dyn_img);

        let mut fields_state = ListState::default();
        fields_state.select_first();

        Ok(Self {
            should_exit: false,
            current_tab: Tab::SeriesList,
            series_list: SeriesList::from_iter(series_list),
            image: protocol,
            comic: ComicState {
                info: ComicInfo::default(),
                fields_state,
                field_inputs: vec![String::new(); 19],
                fields: vec![
                    "title",
                    "series",
                    "number",
                    "volume",
                    "summary",
                    "year",
                    "month",
                    "day",
                    "writer",
                    "penciller",
                    "translator",
                    "publisher",
                    "genre",
                    "tags",
                    "web",
                    "page_count",
                    "language_iso",
                    "manga",
                    "age_rating",
                ],
            },
        })
    }

    /// Run the application
    pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }

        Ok(())
    }

    /// Handle key events
    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != event::KeyEventKind::Press {
            return;
        }

        if self.current_tab == Tab::Metadata {
            match key.code {
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),

                KeyCode::Backspace => {
                    if let Some(sel) = self.comic.fields_state.selected() {
                        self.comic.field_inputs[sel].pop();
                    }
                }
                KeyCode::Delete => {
                    if let Some(sel) = self.comic.fields_state.selected() {
                        self.comic.field_inputs[sel].clear();
                    }
                }

                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.save_inputs_to_info();
                }

                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(sel) = self.comic.fields_state.selected() {
                        self.comic.field_inputs[sel].push(c);
                    }
                }

                KeyCode::Enter => self.current_tab = Tab::ChaptersList,
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,

                // Movement
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('d') => self.select_next_10(),
                KeyCode::Char('u') => self.select_previous_10(),
                KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                KeyCode::Char('G') | KeyCode::End => self.select_last(),
                KeyCode::Char('l') => self.current_tab = Tab::ChaptersList,
                KeyCode::Char('h') => self.current_tab = Tab::SeriesList,
                KeyCode::Enter => {
                    self.current_tab = match self.current_tab {
                        Tab::Metadata => Tab::ChaptersList,
                        Tab::ChaptersList => Tab::Metadata,
                        other @ Tab::SeriesList => other, // leave unchanged
                    };
                }
                _ => {}
            }
        }
    }

    /// Select the next item
    fn select_next(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                self.series_list.state.select_next();
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| series.chapters.state.select_next());
                self.update_chapter_scroll();
            }
            Tab::Metadata => {
                let i = self.comic.fields_state.selected().unwrap_or(0);
                let next = (i + 1) % self.comic.fields.len();
                self.comic.fields_state.select(Some(next));
            }
        }
    }

    /// Select the previous item
    fn select_previous(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                self.series_list.state.select_previous();
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| series.chapters.state.select_previous());
                self.update_chapter_scroll();
            }
            Tab::Metadata => {
                let i = self.comic.fields_state.selected().unwrap_or(0);
                let prev = if i == 0 {
                    self.comic.fields.len() - 1
                } else {
                    i - 1
                };
                self.comic.fields_state.select(Some(prev));
            }
        }
    }

    /// Select `n` items ahead
    fn select_next_n(selected: Option<usize>, n: usize, len: usize) -> usize {
        selected.map_or(len.saturating_add(1), |v| v.saturating_add(n))
    }

    /// Select 10 items ahead
    fn select_next_10(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                let len = self.series_list.items.len();
                let new_idx = Self::select_next_n(self.series_list.state.selected(), 10, len);
                self.series_list.state.select(Some(new_idx));
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| {
                    let len = series.chapters.items.len();
                    let new_idx = Self::select_next_n(series.chapters.state.selected(), 10, len);
                    series.chapters.state.select(Some(new_idx));
                });
                self.update_chapter_scroll();
            }
            Tab::Metadata => {}
        }
    }

    /// Select `n` items behind
    fn select_previous_n(selected: Option<usize>, n: usize, len: usize) -> usize {
        selected.map_or(len.saturating_sub(1), |v| v.saturating_sub(n))
    }

    /// Select 10 items behind
    fn select_previous_10(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                let len = self.series_list.items.len();
                let new_idx = Self::select_previous_n(self.series_list.state.selected(), 10, len);
                self.series_list.state.select(Some(new_idx));
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| {
                    let len = series.chapters.items.len();
                    let new_idx =
                        Self::select_previous_n(series.chapters.state.selected(), 10, len);
                    series.chapters.state.select(Some(new_idx));
                });
                self.update_chapter_scroll();
            }
            Tab::Metadata => {}
        }
    }

    /// Select the first item
    fn select_first(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                self.series_list.state.select_first();
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| series.chapters.state.select_first());
                self.update_chapter_scroll();
            }
            Tab::Metadata => {}
        }
    }

    /// Select the last item
    fn select_last(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                self.series_list.state.select_last();
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| series.chapters.state.select_last());
                self.update_chapter_scroll();
            }
            Tab::Metadata => {}
        }
    }

    fn update_series_scroll(&mut self) {
        let current = self.series_list.state.selected().unwrap_or_default();
        self.series_list.scroll_state = self.series_list.scroll_state.position(current);
    }

    fn update_chapter_scroll(&mut self) {
        let current = self.series_list.state.selected().unwrap_or_default();
        if let Some(series) = self.series_list.items_state.get_mut(current) {
            let current_chapter = series.chapters.state.selected().unwrap_or(0);
            series.chapters.scroll_state = series.chapters.scroll_state.position(current_chapter);
        }
    }

    /// Update the chapter select
    ///
    /// Updates the current chapter path and the comic info
    /// based on the selected chapter
    fn update_chapter_select(&mut self, select: fn(&mut Series)) {
        let current = self.series_list.state.selected().unwrap_or_default();
        let current_chapter_path = {
            if let Some(series) = self.series_list.items_state.get_mut(current) {
                select(series);
                let current_chapter = series.chapters.state.selected().unwrap_or(0);
                series
                    .chapters
                    .items
                    .get(current_chapter)
                    .map(|c| c.path.clone())
            } else {
                None
            }
        };

        self.update_comic_info(current_chapter_path);
    }

    /// Update the comic info
    ///
    /// Updates the comic info based on the chapter path
    fn update_comic_info(&mut self, chapter_path: Option<PathBuf>) {
        if let Some(path) = chapter_path {
            self.comic.info = get_comic_from_zip(&path).unwrap_or_default();
            self.sync_inputs_from_info();
        }
    }
}

impl App {
    /// Sync the inputs from the [`ComicInfo`]
    fn sync_inputs_from_info(&mut self) {
        for (i, name) in self.comic.fields.iter().enumerate() {
            self.comic.field_inputs[i] = match *name {
                "title" => self.comic.info.title.clone(),
                "series" => self.comic.info.series.clone(),
                "number" => self
                    .comic
                    .info
                    .number
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                "volume" => self
                    .comic
                    .info
                    .volume
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                "summary" => self.comic.info.summary.clone().unwrap_or_default(),
                "year" => self
                    .comic
                    .info
                    .year
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                "month" => self
                    .comic
                    .info
                    .month
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                "day" => self
                    .comic
                    .info
                    .day
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                "writer" => self.comic.info.writer.clone().unwrap_or_default(),
                "penciller" => self.comic.info.penciller.clone().unwrap_or_default(),
                "translator" => self.comic.info.translator.clone().unwrap_or_default(),
                "publisher" => self.comic.info.publisher.clone().unwrap_or_default(),
                "genre" => self.comic.info.genre.clone().unwrap_or_default(),
                "tags" => self.comic.info.tags.clone().unwrap_or_default(),
                "web" => self.comic.info.web.clone().unwrap_or_default(),
                "page_count" => self
                    .comic
                    .info
                    .page_count
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                "language_iso" => self.comic.info.language_iso.clone().unwrap_or_default(),
                "manga" => self.comic.info.manga.to_string().clone(),
                "age_rating" => self.comic.info.age_rating.to_string().clone(),
                _ => String::new(),
            };
        }
    }

    /// Save the inputs to the [`ComicInfo`]
    fn save_inputs_to_info(&mut self) {
        for (i, name) in self.comic.fields.iter().enumerate() {
            let input = self.comic.field_inputs[i].trim();
            match *name {
                "title" => self.comic.info.title = input.to_string(),
                "series" => self.comic.info.series = input.to_string(),
                "number" => self.comic.info.number = input.parse().ok(),
                "volume" => self.comic.info.volume = input.parse().ok(),
                "summary" => self.comic.info.summary = Some(input.to_string()),
                "year" => self.comic.info.year = input.parse().ok(),
                "month" => self.comic.info.month = input.parse().ok(),
                "day" => self.comic.info.day = input.parse().ok(),
                "writer" => self.comic.info.writer = Some(input.to_string()),
                "penciller" => self.comic.info.penciller = Some(input.to_string()),
                "translator" => self.comic.info.translator = Some(input.to_string()),
                "publisher" => self.comic.info.publisher = Some(input.to_string()),
                "genre" => self.comic.info.genre = Some(input.to_string()),
                "tags" => self.comic.info.tags = Some(input.to_string()),
                "web" => self.comic.info.web = Some(input.to_string()),
                "page_count" => self.comic.info.page_count = input.parse().ok(),
                "language_iso" => self.comic.info.language_iso = Some(input.to_string()),
                "manga" => self.comic.info.manga = input.to_string().into(),
                "age_rating" => self.comic.info.age_rating = input.to_string().into(),
                _ => {}
            }
        }

        // TODO: Remove
        std::fs::write("test.txt", format!("{:#?}", self.comic.info)).unwrap();
    }
}

impl App {
    fn get_current_series(&self) -> Series {
        let current = self.series_list.state.selected().unwrap_or_default();
        self.series_list.items_state[current].clone()
    }

    fn get_current_chapter(&self) -> Chapter {
        let series = self.get_current_series();
        let current = series.chapters.state.selected().unwrap_or_default();
        series.chapters.items_state[current].clone()
    }
}
