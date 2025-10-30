use std::time::Instant;

use crate::{
    chapter_manager::{
        save_chapter_info, save_series_info, update_chapter_numbering, update_volume_numbering,
    },
    managers::comic_form::ComicFormState,
    ui::{App, InputMode, Tab, list::Series},
};

/// Handles keybindings in metadata tab
impl App {
    pub fn handle_ctrl_d(&self) {
        let ComicFormState::Ready(comic) = &self.comic_manager.comic else {
            return;
        };
        let chapters = self.get_chapters_in_series();
        let comic_info = comic.to_comic_info();
        let status_tx = self.status_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = save_series_info(chapters, comic_info, status_tx).await {
                error!("Failed to save series info: {e}");
            }
        });
    }

    pub fn handle_ctrl_s(&self) {
        let ComicFormState::Ready(comic) = &self.comic_manager.comic else {
            return;
        };

        let chapter = self.get_current_chapter();
        let comic_info = comic.to_comic_info();
        let status_tx = self.status_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = save_chapter_info(chapter, comic_info, status_tx).await {
                error!("Failed to save chapter info: {e}");
            }
        });
    }

    pub fn handle_ctrl_f(&self) {
        if let ComicFormState::Ready(_) = &self.comic_manager.comic {
            let chapters = self.get_chapters_in_series();
            let status_tx = self.status_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = update_chapter_numbering(chapters, status_tx).await {
                    error!("Failed to save series info: {e}");
                }
            });
        }
    }

    pub fn handle_ctrl_g(&self) {
        let ComicFormState::Ready(comic) = &self.comic_manager.comic else {
            return;
        };

        let chapters = self.get_chapters_in_series();
        let comic_info = comic.to_comic_info();
        let status_tx = self.status_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = update_volume_numbering(chapters, comic_info, status_tx).await {
                error!("Failed to save series info: {e}");
            }
        });
    }

    pub fn handle_ctrl_u(&self) {
        let ComicFormState::Ready(comic) = &self.comic_manager.comic else {
            error!("Comic is not ready");
            return;
        };

        let path = self.get_current_series().path;
        // TODO: Get this from config
        let path = if path.ends_with("_oneshots") {
            self.get_current_chapter().path
        } else {
            path
        };

        let komga_manager = self.komga_manager.clone();
        tokio::spawn(async move {
            if let Ok(series) = komga_manager.list_series().await
                && let Some(series) = series
                    .content
                    .iter()
                    .find(|v| v.url == path.to_string_lossy())
            {
                debug!("Found series: {series:?}");
                let Ok(books) = komga_manager.list_books(&series.id).await else {
                    error!("Failed to list books for series ({})", path.display());
                    return;
                };

                info!("{books:#?}");
            } else {
                error!("Failed to find series ({})", path.display());
            }
        });
    }

    /// Clears the chapter selection
    pub fn handle_esc_selection(&mut self) {
        let current = self.series_list.state.selected().unwrap_or_default();
        if let Some(series) = self.series_list.items_state.get_mut(current) {
            series.chapters.selected.clear();
        }
    }

    /// Changes input to normal
    pub fn handle_esc_editing(&mut self) {
        if self.input_mode == InputMode::Editing {
            self.input_mode = InputMode::Normal;
        } else {
            self.current_tab = Tab::ChaptersList;
        }
    }
}

impl App {
    pub fn next_tab(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.current_tab = Tab::ChaptersList,
            Tab::ChaptersList => self.current_tab = Tab::Metadata,
            Tab::Metadata => {}
        }
    }

    pub fn previous_tab(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.current_tab = Tab::ChaptersList,
            Tab::ChaptersList => self.current_tab = Tab::SeriesList,
            Tab::Metadata => {}
        }
    }

    /// Select the next item
    pub fn select_next(&mut self) {
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
    pub fn select_previous(&mut self) {
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
    pub fn select_next_10(&mut self) {
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
    pub fn select_previous_10(&mut self) {
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
    pub fn select_first(&mut self) {
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
    pub fn select_last(&mut self) {
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

    /// Toggle the selection of the current chapter
    pub fn toggle_select(&mut self) {
        let current = self.series_list.state.selected().unwrap_or_default();
        if let Some(series) = self.series_list.items_state.get_mut(current) {
            series.chapters.toggle_selected();
            self.select_next();
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
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

        self.comic_manager.comic = ComicFormState::Loading;
        self.last_selection_change = Some(Instant::now());
        self.pending_selection = current_chapter_path;
    }
}
