//! UI for the application

use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

use futures::{StreamExt, stream};
use image::ImageReader;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::watch;
use tui_input::backend::crossterm::EventHandler;

use crate::{
    comic_info::ComicInfo,
    ui::{
        comic_form::{ComicFormState, ComicInfoForm},
        list::{Chapter, Series, SeriesList},
        spinner::SpinnerState,
    },
    zip_util::{get_comic_from_zip, modify_zip_in_memory},
};

pub mod app;
pub mod comic_form;
pub mod list;
pub mod spinner;

/// Debounce delay for chapter selection
const TICK_RATE: Duration = Duration::from_millis(100);

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

/// Main application
pub struct App {
    should_exit: bool,
    current_tab: Tab,
    series_list: SeriesList,
    image: StatefulProtocol,

    input_mode: InputMode,

    comic: ComicFormState,
    comic_rx: Option<mpsc::Receiver<ComicInfoForm>>,
    last_selection_change: Option<Instant>,
    pending_selection: Option<PathBuf>,
    spinner: SpinnerState,
    status_rx: watch::Receiver<String>,
    status_tx: watch::Sender<String>,
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

        let (status_tx, status_rx) = watch::channel("Idle".to_string());

        Ok(Self {
            should_exit: false,
            current_tab: Tab::SeriesList,
            series_list: SeriesList::from_iter(series_list),
            image: protocol,
            comic: ComicFormState::Loading(()),
            input_mode: InputMode::Normal,
            comic_rx: None,
            last_selection_change: None,
            pending_selection: None,
            spinner: SpinnerState::default(),
            status_rx,
            status_tx,
        })
    }

    /// Run the application
    pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        let mut last_tick = std::time::Instant::now();
        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
            {
                self.handle_key(key);
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.tick();
                last_tick = std::time::Instant::now();
            }
        }

        Ok(())
    }

    fn tick(&mut self) {
        // check for finished async loads
        self.poll_comic_info();

        // debounce loading
        if let Some(path) = self.pending_selection.clone() {
            // start background load after 0.5s idle
            self.update_comic_info(Some(path));
            self.pending_selection = None;
        }

        self.spinner.tick();
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
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_exit = true;
                }

                // Movement
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('d') | KeyCode::PageDown => self.select_next_10(),
                KeyCode::Char('u') | KeyCode::PageUp => self.select_previous_10(),
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
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let ComicFormState::Ready(comic) = &self.comic {
                    let chapters = self.get_chapters_in_series();
                    let comic_info = comic.to_comic_info();
                    let status_tx = self.status_tx.clone();

                    tokio::spawn(async move {
                        if let Err(e) = save_series_info(chapters, comic_info, status_tx).await {
                            eprintln!("Failed to save series info: {e}");
                        }
                    });
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let ComicFormState::Ready(comic) = &self.comic {
                    let chapter = self.get_current_chapter();
                    let comic_info = comic.to_comic_info();
                    let status_tx = self.status_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = save_chapter_info(chapter, comic_info, status_tx).await {
                            eprintln!("Failed to save series info: {e}");
                        }
                    });
                }
            }
            KeyCode::Enter if self.input_mode == InputMode::Normal => {
                self.input_mode = InputMode::Editing;
            }
            KeyCode::Enter if self.input_mode == InputMode::Editing => {
                self.input_mode = InputMode::Normal;
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
                if self.input_mode == InputMode::Editing
                    && let Some(input) = self.comic.active_input_mut()
                {
                    input.handle_event(&Event::Key(key));
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
                self.update_chapter_select(|series| {
                    series.chapters.state.selected();
                });
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
                self.update_chapter_select(|series| {
                    series.chapters.state.selected();
                });
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
                self.update_chapter_select(|series| {
                    series.chapters.state.selected();
                });
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
                self.update_chapter_select(|series| {
                    series.chapters.state.selected();
                });
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

        self.comic = ComicFormState::Loading(());
        self.last_selection_change = Some(Instant::now());
        self.pending_selection = current_chapter_path;
    }
}

impl App {
    /// Update the comic info
    ///
    /// Updates the comic info based on the chapter path
    fn update_comic_info(&mut self, chapter_path: Option<PathBuf>) {
        if let Some(path) = chapter_path {
            let (tx, rx) = std::sync::mpsc::channel();
            self.comic_rx = Some(rx);
            self.comic = ComicFormState::Loading(());

            std::thread::spawn(move || {
                let info = get_comic_from_zip(&path).unwrap_or_default();
                let form = ComicInfoForm::new(&info);
                let _ = tx.send(form);
            });
        }
    }

    fn poll_comic_info(&mut self) {
        if let Some(rx) = &self.comic_rx
            && let Ok(form) = rx.try_recv()
        {
            self.comic = ComicFormState::Ready(form);
            self.comic_rx = None;
        }
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

    fn get_chapters_in_series(&self) -> Vec<Chapter> {
        let series = self.get_current_series();
        series.chapters.items_state
    }
}

/// Save the inputs to the [`ComicInfo`]
pub async fn save_chapter_info(
    chapter: Chapter,
    comic_info: ComicInfo,
    status_tx: watch::Sender<String>,
) -> anyhow::Result<()> {
    let total_start = Instant::now();
    let path = chapter.path.clone();
    let title = chapter
        .title
        .clone()
        .unwrap_or_else(|| path.display().to_string());
    let _ = status_tx.send(format!("Processing: {title}"));

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let path = chapter.path;
        let xml = quick_xml::se::to_string(&comic_info)?;
        modify_zip_in_memory(&path, xml.as_bytes())?;

        Ok(())
    })
    .await??;

    let total_duration = total_start.elapsed();
    let _ = status_tx.send(format!(
        "All done~ processed chapter in {total_duration:.2?} 🎉"
    ));
    Ok(())
}

pub async fn save_series_info(
    chapters: Vec<Chapter>,
    comic_info: ComicInfo,
    status_tx: watch::Sender<String>,
) -> anyhow::Result<()> {
    let chapters_len = chapters.len();
    // TODO: Make this in config
    let concurrency_limit = num_cpus::get();
    let total_start = Instant::now();

    stream::iter(chapters.into_iter().enumerate())
        .map(|(i, chapter)| {
            let status_tx = status_tx.clone();
            let info = comic_info.clone();

            async move {
                let path = chapter.path.clone();
                let title = chapter
                    .title
                    .clone()
                    .unwrap_or_else(|| path.display().to_string());

                let _ = status_tx.send(format!("Processing {}/{}: {}", i + 1, chapters_len, title));

                // run CPU-heavy / sync work
                tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                    // TODO: Get ComicInfo while modifing
                    let mut old = get_comic_from_zip(&path).unwrap_or_default();
                    old.update_shared_fields(&info);
                    let xml = quick_xml::se::to_string(&old)?;
                    modify_zip_in_memory(&path, xml.as_bytes())?;
                    Ok(())
                })
                .await??;

                Ok::<_, anyhow::Error>(())
            }
        })
        .buffer_unordered(concurrency_limit)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    let total_duration = total_start.elapsed();

    let _ = status_tx.send(format!(
        "All done~ processed {chapters_len} chapters in {total_duration:.2?} 🎉"
    ));

    Ok(())
}
