//! UI for the application

use std::path::PathBuf;

use image::ImageReader;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    comic_info::{ComicInfo, ComicInfoAgeRating, ComicInfoManga},
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

/// Current input mode
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

/// Current comic selected on chapter list
pub struct ComicInfoForm {
    fields: Vec<(&'static str, Input)>, // label + input
    active_index: usize,
}

impl ComicInfoForm {
    pub fn new(info: &ComicInfo) -> Self {
        let fields = vec![
            ("Title", Input::new(info.title.clone())),
            ("Series", Input::new(info.series.clone())),
            (
                "Number",
                Input::new(info.number.map(|n| n.to_string()).unwrap_or_default()),
            ),
            (
                "Volume",
                Input::new(info.volume.map(|v| v.to_string()).unwrap_or_default()),
            ),
            (
                "Summary",
                Input::new(info.summary.clone().unwrap_or_default()),
            ),
            (
                "Year",
                Input::new(info.year.map(|y| y.to_string()).unwrap_or_default()),
            ),
            (
                "Month",
                Input::new(info.month.map(|m| m.to_string()).unwrap_or_default()),
            ),
            (
                "Day",
                Input::new(info.day.map(|d| d.to_string()).unwrap_or_default()),
            ),
            (
                "Writer",
                Input::new(info.writer.clone().unwrap_or_default()),
            ),
            (
                "Penciller",
                Input::new(info.penciller.clone().unwrap_or_default()),
            ),
            (
                "Translator",
                Input::new(info.translator.clone().unwrap_or_default()),
            ),
            (
                "Publisher",
                Input::new(info.publisher.clone().unwrap_or_default()),
            ),
            ("Genre", Input::new(info.genre.clone().unwrap_or_default())),
            ("Tags", Input::new(info.tags.clone().unwrap_or_default())),
            ("Web", Input::new(info.web.clone().unwrap_or_default())),
            (
                "Page Count",
                Input::new(info.page_count.map(|p| p.to_string()).unwrap_or_default()),
            ),
            (
                "Language ISO",
                Input::new(info.language_iso.clone().unwrap_or_default()),
            ),
            ("Manga", Input::new(format!("{:?}", info.manga))),
            ("Age Rating", Input::new(format!("{:?}", info.age_rating))),
        ];

        Self {
            fields,
            active_index: 0,
        }
    }

    pub fn next(&mut self) {
        self.active_index = (self.active_index + 1) % self.fields.len();
    }

    pub fn next_side(&mut self) {
        self.active_index = (self.active_index + 10) % self.fields.len();
    }

    pub fn prev(&mut self) {
        if self.active_index == 0 {
            self.active_index = self.fields.len() - 1;
        } else {
            self.active_index -= 1;
        }
    }

    pub fn prev_side(&mut self) {
        let step = 10 % self.fields.len();
        if self.active_index < step {
            self.active_index = self.fields.len() + self.active_index - step;
        } else {
            self.active_index -= step;
        }
    }

    pub fn active_input_mut(&mut self) -> &mut Input {
        &mut self.fields[self.active_index].1
    }

    pub fn to_comic_info(&self) -> ComicInfo {
        ComicInfo {
            title: self.fields[0].1.value().to_string(),
            series: self.fields[1].1.value().to_string(),
            number: parse_opt_f32(self.fields[2].1.value()),
            volume: parse_opt_u32(self.fields[3].1.value()),
            summary: parse_opt_string(self.fields[4].1.value()),
            year: parse_opt_u16(self.fields[5].1.value()),
            month: parse_opt_u16(self.fields[6].1.value()),
            day: parse_opt_u8(self.fields[7].1.value()),
            writer: parse_opt_string(self.fields[8].1.value()),
            penciller: parse_opt_string(self.fields[9].1.value()),
            translator: parse_opt_string(self.fields[10].1.value()),
            publisher: parse_opt_string(self.fields[11].1.value()),
            genre: parse_opt_string(self.fields[12].1.value()),
            tags: parse_opt_string(self.fields[13].1.value()),
            web: parse_opt_string(self.fields[14].1.value()),
            page_count: parse_opt_u32(self.fields[15].1.value()),
            language_iso: parse_opt_string(self.fields[16].1.value()),
            manga: parse_enum::<ComicInfoManga>(self.fields[17].1.value()).unwrap_or_default(),
            age_rating: parse_enum::<ComicInfoAgeRating>(self.fields[18].1.value())
                .unwrap_or_default(),
        }
    }
}

fn parse_opt_string(s: &str) -> Option<String> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn parse_opt_f32(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok()
}

fn parse_opt_u32(s: &str) -> Option<u32> {
    s.trim().parse::<u32>().ok()
}

fn parse_opt_u16(s: &str) -> Option<u16> {
    s.trim().parse::<u16>().ok()
}

fn parse_opt_u8(s: &str) -> Option<u8> {
    s.trim().parse::<u8>().ok()
}

// For enum fields like Manga and AgeRating
fn parse_enum<T: std::str::FromStr>(s: &str) -> Option<T> {
    s.trim().parse::<T>().ok()
}

/// Main application
pub struct App {
    should_exit: bool,
    current_tab: Tab,
    series_list: SeriesList,
    image: StatefulProtocol,

    input_mode: InputMode,

    comic: ComicInfoForm,
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
        let picker = Picker::from_query_stdio()?;
        let protocol = picker.new_resize_protocol(dyn_img);

        let mut fields_state = ListState::default();
        fields_state.select_first();

        Ok(Self {
            should_exit: false,
            current_tab: Tab::SeriesList,
            series_list: SeriesList::from_iter(series_list),
            image: protocol,
            comic: ComicInfoForm::new(&ComicInfo::default()),
            input_mode: InputMode::Normal,
        })
    }

    /// Run the application
    pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;

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
            self.handle_key_metadata(key);
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
                KeyCode::Char('l') | KeyCode::Enter => self.next_tab(),
                KeyCode::Char('h') => self.previous_tab(),
                _ => {}
            }
        }
    }

    fn handle_key_metadata(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_inputs_to_info();
            }
            KeyCode::Char('e') if self.input_mode == InputMode::Normal => {
                self.input_mode = InputMode::Editing;
            }
            KeyCode::Char('j') | KeyCode::Tab if self.input_mode == InputMode::Normal => {
                self.comic.next();
            }
            KeyCode::Char('k') | KeyCode::BackTab if self.input_mode == InputMode::Normal => {
                self.comic.prev();
            }
            KeyCode::Char('l') if self.input_mode == InputMode::Normal => self.comic.next_side(),
            KeyCode::Char('h') if self.input_mode == InputMode::Normal => self.comic.prev_side(),

            KeyCode::Esc => {
                if self.input_mode == InputMode::Editing {
                    self.input_mode = InputMode::Normal;
                } else {
                    self.current_tab = Tab::ChaptersList;
                }
            }
            _ => {
                if self.input_mode == InputMode::Editing {
                    self.comic.active_input_mut().handle_event(&Event::Key(key));
                }
            }
        }
    }

    fn next_tab(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.current_tab = Tab::ChaptersList,
            Tab::ChaptersList => self.current_tab = Tab::Metadata,
            Tab::Metadata => {}
        }
    }

    fn previous_tab(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.current_tab = Tab::ChaptersList,
            Tab::ChaptersList => self.current_tab = Tab::SeriesList,
            Tab::Metadata => {}
        }
    }

    /// Select the next item
    fn select_next(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                self.series_list.state.select_next();
                self.update_chapter_select(|series| {
                    series.chapters.state.selected();
                });
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| series.chapters.state.select_next());
                self.update_chapter_scroll();
            }
            Tab::Metadata => {}
        }
    }

    /// Select the previous item
    fn select_previous(&mut self) {
        match self.current_tab {
            Tab::SeriesList => {
                self.series_list.state.select_previous();
                self.update_chapter_select(|series| {
                    series.chapters.state.selected();
                });
                self.update_series_scroll();
            }
            Tab::ChaptersList => {
                self.update_chapter_select(|series| series.chapters.state.select_previous());
                self.update_chapter_scroll();
            }
            Tab::Metadata => {}
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

    /// Update the series scroll
    fn update_series_scroll(&mut self) {
        let current = self.series_list.state.selected().unwrap_or_default();
        self.series_list.scroll_state = self.series_list.scroll_state.position(current);
    }

    /// Update the chapter scroll
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
}

impl App {
    /// Update the comic info
    ///
    /// Updates the comic info based on the chapter path
    fn update_comic_info(&mut self, chapter_path: Option<PathBuf>) {
        if let Some(path) = chapter_path {
            self.comic = ComicInfoForm::new(&get_comic_from_zip(&path).unwrap_or_default());
            // self.sync_inputs_from_info();
        }
    }

    /// Save the inputs to the [`ComicInfo`]
    fn save_inputs_to_info(&mut self) {
        // TODO: Remove
        std::fs::write("test.txt", format!("{:#?}", self.comic.to_comic_info()))
            .unwrap_or_default();
    }

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
