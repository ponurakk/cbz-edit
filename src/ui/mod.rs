//! UI for the application

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};
use ratatui_image::picker::Picker;
use tokio::sync::watch;
use tui_input::backend::crossterm::EventHandler;

use crate::{
    config::Config,
    managers::{
        comic_form::{ComicFormState, ComicInfoForm, ComicInfoManager},
        image::{ImageManager, ImagesState},
        komga::KomgaManager,
    },
    ui::list::{Chapter, Series, SeriesList},
    zip_util::get_comic_from_zip,
};

pub mod app;
pub mod components;
pub mod keybindings;
pub mod list;
pub mod widgets;

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
    /// Exit flag
    should_exit: bool,

    /// Current Selected Tab
    current_tab: Tab,

    /// List of the series in the library
    series_list: SeriesList,

    /// Images in currently selected chapter
    image_manager: ImageManager,

    /// Komga manager
    komga_manager: KomgaManager,

    /// Comic form state
    comic_manager: ComicInfoManager,

    /// Help flag
    show_help: bool,

    /// Current input mode
    input_mode: InputMode,

    /// Last time the selection changed
    last_selection_change: Option<Instant>,

    /// Pending selection to update
    pending_selection: Option<PathBuf>,

    /// Reciever channel for status
    status_rx: watch::Receiver<String>,

    /// Sender channel for status
    status_tx: watch::Sender<String>,
}

impl Default for App {
    fn default() -> Self {
        Self::new(vec![], &Config::default()).expect("Failed to create app")
    }
}

impl App {
    /// Create a new application
    pub fn new(series_list: Vec<Series>, config: &Config) -> anyhow::Result<Self> {
        let picker = Picker::from_query_stdio()?;

        let mut fields_state = ListState::default();
        fields_state.select_first();

        let (status_tx, status_rx) = watch::channel("Idle".to_string());

        Ok(Self {
            should_exit: false,
            current_tab: Tab::SeriesList,
            series_list: SeriesList::from_iter(series_list),
            image_manager: ImageManager::new(picker),
            komga_manager: KomgaManager::new(&config.komga.url, &config.komga.api_key)?,
            comic_manager: ComicInfoManager::new(),
            show_help: false,
            input_mode: InputMode::Normal,
            last_selection_change: None,
            pending_selection: None,
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
        self.poll_images();
        self.image_manager.poll_image_updates();

        // debounce loading
        if let Some(path) = self.pending_selection.clone() {
            // start background load after 0.5s idle
            self.update_comic_info(Some(path));
            self.pending_selection = None;
        }

        self.comic_manager.spinner.tick();
        self.image_manager.spinner.tick();
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
                KeyCode::Char(' ') if self.current_tab == Tab::ChaptersList => self.toggle_select(),
                KeyCode::Char('?') => self.toggle_help(),
                KeyCode::Char('=' | '+') => self.image_manager.next(),
                KeyCode::Char('-') => self.image_manager.prev(),
                _ => {}
            }
        }
    }

    fn handle_key_metadata(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_ctrl_d();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_ctrl_s();
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_ctrl_f();
            }
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_ctrl_g();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_ctrl_u();
            }
            KeyCode::Enter if self.input_mode == InputMode::Normal => {
                self.input_mode = InputMode::Editing;
            }
            KeyCode::Enter if self.input_mode == InputMode::Editing => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Tab if self.input_mode == InputMode::Normal => {
                self.comic_manager.comic.next();
            }
            KeyCode::Char('k') | KeyCode::BackTab if self.input_mode == InputMode::Normal => {
                self.comic_manager.comic.prev();
            }
            KeyCode::Char('l') if self.input_mode == InputMode::Normal => {
                self.comic_manager.comic.next_side();
            }
            KeyCode::Char('h') if self.input_mode == InputMode::Normal => {
                self.comic_manager.comic.prev_side();
            }
            KeyCode::Char('=' | '+') => self.image_manager.next(),
            KeyCode::Char('-') => self.image_manager.prev(),
            KeyCode::Esc => self.handle_esc(),
            _ => {
                if self.input_mode == InputMode::Editing
                    && let Some(input) = self.comic_manager.comic.active_input_mut()
                {
                    input.handle_event(&Event::Key(key));
                }
            }
        }
    }
}

impl App {
    /// Update the comic info
    ///
    /// Updates the comic info based on the chapter path
    fn update_comic_info(&mut self, chapter_path: Option<PathBuf>) {
        if let Some(path) = chapter_path {
            let (comic_tx, comic_rx) = std::sync::mpsc::channel();
            self.comic_manager.comic_rx = Some(comic_rx);
            self.comic_manager.comic = ComicFormState::Loading;

            let (images_tx, images_rx) = std::sync::mpsc::channel();
            self.image_manager.raw_images_rx = Some(images_rx);
            self.image_manager.images = ImagesState::Loading;

            #[allow(clippy::cast_possible_truncation)]
            tokio::spawn(async move {
                let (mut info, images) = get_comic_from_zip(&path).unwrap_or_default();
                info.page_count = Some(images.len() as u32);
                let form = ComicInfoForm::new(&info);
                let _ = comic_tx.send(form);
                let _ = images_tx.send(images);
            });
        }
    }

    fn poll_comic_info(&mut self) {
        if let Some(rx) = &self.comic_manager.comic_rx
            && let Ok(form) = rx.try_recv()
        {
            self.comic_manager.comic = ComicFormState::Ready(form);
            self.comic_manager.comic_rx = None;
        }
    }

    fn poll_images(&mut self) {
        if let Some(rx) = &self.image_manager.raw_images_rx
            && let Ok(images) = rx.try_recv()
        {
            self.image_manager.replace_images(images);
            self.image_manager.raw_images_rx = None;
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

        if series.chapters.selected.is_empty() {
            series.chapters.items_state.clone()
        } else {
            series
                .chapters
                .selected
                .iter()
                .filter_map(|&i| series.chapters.items_state.get(i).cloned())
                .collect()
        }
    }
}
