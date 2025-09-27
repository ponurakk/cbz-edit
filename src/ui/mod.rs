//! UI for the application

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
};

use crate::ui::list::{Series, SeriesList};

pub mod app;
pub mod list;

#[derive(PartialEq, Eq)]
pub enum Tab {
    SeriesList,
    ChaptersList,
}

pub struct App {
    should_exit: bool,
    current_tab: Tab,
    series_list: SeriesList,
}

impl Default for App {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl App {
    pub fn new(series_list: Vec<Series>) -> Self {
        Self {
            should_exit: false,
            current_tab: Tab::SeriesList,
            series_list: SeriesList::from_iter(series_list),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != event::KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            // Movement
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('d') => self.select_next_10(),
            KeyCode::Char('u') => self.select_previous_10(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Tab => {
                if self.current_tab == Tab::SeriesList {
                    self.current_tab = Tab::ChaptersList;
                } else if self.current_tab == Tab::ChaptersList {
                    self.current_tab = Tab::SeriesList;
                }
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.series_list.state.select_next(),
            Tab::ChaptersList => {
                let current = self.series_list.state.selected().unwrap_or_default();
                if let Some(series) = self.series_list.items_state.get_mut(current) {
                    series.chapters.state.select_next();
                }
            }
        }
    }

    fn select_previous(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.series_list.state.select_previous(),
            Tab::ChaptersList => {
                let current = self.series_list.state.selected().unwrap_or_default();
                if let Some(series) = self.series_list.items_state.get_mut(current) {
                    series.chapters.state.select_previous();
                }
            }
        }
    }

    fn select_next_10(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.series_list.state.select(Some(
                self.series_list.state.selected().map_or(0, |v| v + 10),
            )),
            Tab::ChaptersList => {
                let current = self.series_list.state.selected().unwrap_or_default();
                if let Some(series) = self.series_list.items_state.get_mut(current) {
                    series
                        .chapters
                        .state
                        .select(Some(series.chapters.state.selected().map_or(0, |v| v + 10)));
                }
            }
        }
    }

    fn select_previous_10(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.series_list.state.select(Some(
                self.series_list
                    .state
                    .selected()
                    .map_or(self.series_list.items.len(), |v| v.saturating_sub(10)),
            )),
            Tab::ChaptersList => {
                let current = self.series_list.state.selected().unwrap_or_default();
                if let Some(series) = self.series_list.items_state.get_mut(current) {
                    series.chapters.state.select(Some(
                        series
                            .chapters
                            .state
                            .selected()
                            .map_or(series.chapters.items.len(), |v| v.saturating_sub(10)),
                    ));
                }
            }
        }
    }

    fn select_first(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.series_list.state.select_first(),
            Tab::ChaptersList => {
                let current = self.series_list.state.selected().unwrap_or_default();
                if let Some(series) = self.series_list.items_state.get_mut(current) {
                    series.chapters.state.select_first();
                }
            }
        }
    }

    fn select_last(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.series_list.state.select_last(),
            Tab::ChaptersList => {
                let current = self.series_list.state.selected().unwrap_or_default();
                if let Some(series) = self.series_list.items_state.get_mut(current) {
                    series.chapters.state.select_last();
                }
            }
        }
    }
}
