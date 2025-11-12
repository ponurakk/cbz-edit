use std::time::Instant;

use crate::{
    chapter_manager::{
        save_chapter_info, save_series_info, update_chapter_numbering, update_volume_numbering,
    },
    managers::comic_form::{ComicFormState, ComicInfoForm},
    ui::{
        App, InputMode, Tab,
        list::{ChapterList, Series},
    },
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

    pub fn handle_ctrl_u(&mut self) {
        let ComicFormState::Ready(comic) = &self.comic_manager.comic else {
            error!("Comic is not ready");
            return;
        };
        let comic_info = comic.to_comic_info();

        let series_path = self.get_current_series().path;
        let chapter_path = self.get_current_chapter().path;
        let series_path = if series_path.ends_with(&self.config.komga.oneshots_dir) {
            chapter_path.clone()
        } else {
            series_path
        };

        let (comic_tx, comic_rx) = std::sync::mpsc::channel();
        self.comic_manager.comic_rx = Some(comic_rx);
        self.comic_manager.comic = ComicFormState::Loading;

        let status_tx = self.status_tx.clone();
        let komga_manager = self.komga_manager.clone();
        tokio::spawn(async move {
            let Ok(series) = komga_manager.list_series().await else {
                return error!("Failed to list series ({})", series_path.display());
            };

            let Some(series) = series
                .content
                .iter()
                .find(|v| v.url == series_path.to_string_lossy())
            else {
                return error!("Failed to find series ({})", series_path.display());
            };
            debug!("Found series: {series:?}");

            let books = match komga_manager.list_books(&series.id).await {
                Ok(books) => books,
                Err(e) => {
                    return error!(
                        "Failed to list books for series ({}) with error: {e:?}",
                        series_path.display(),
                    );
                }
            };

            let Some(book) = books
                .content
                .iter()
                .find(|book| book.url == chapter_path.to_string_lossy())
            else {
                let _ = status_tx.send(format!("Failed to find book ({})", chapter_path.display()));
                return error!("Failed to find book ({})", chapter_path.display());
            };

            let info_form = book.to_comic_info(series, &comic_info);
            let form = ComicInfoForm::new(&info_form);
            let _ = comic_tx.send(form);
        });
    }

    pub fn handle_ctrl_a(&self) {
        let ComicFormState::Ready(_) = &self.comic_manager.comic else {
            error!("Comic is not ready");
            return;
        };

        let series_path = self.get_current_series().path;
        let series_path = if series_path.ends_with(&self.config.komga.oneshots_dir) {
            self.get_current_chapter().path
        } else {
            series_path
        };

        let komga_manager = self.komga_manager.clone();
        let status_tx = self.status_tx.clone();
        tokio::spawn(async move {
            let Ok(series) = komga_manager.list_series().await else {
                return error!("Failed to list series ({})", series_path.display());
            };

            let Some(series) = series
                .content
                .iter()
                .find(|v| v.url == series_path.to_string_lossy())
            else {
                return error!("Failed to find series ({})", series_path.display());
            };
            debug!("Found series: {series:?}");

            if let Err(e) = komga_manager.analyze_series(&series.id).await {
                error!(
                    "Failed to analyze series ({}) with error: {e}",
                    series_path.display()
                );
            }

            let _ = status_tx.send(format!("Sent analyze request for series ({})", series.name));
        });
    }

    pub fn handle_ctrl_q(&self) {
        let ComicFormState::Ready(_) = &self.comic_manager.comic else {
            error!("Comic is not ready");
            return;
        };

        let series_path = self.get_current_series().path;
        let series_path = if series_path.ends_with(&self.config.komga.oneshots_dir) {
            self.get_current_chapter().path
        } else {
            series_path
        };

        let komga_manager = self.komga_manager.clone();
        let komf_manager = self.komf_manager.clone();
        let status_tx = self.status_tx.clone();
        tokio::spawn(async move {
            let Ok(series) = komga_manager.list_series().await else {
                return error!("Failed to list series ({})", series_path.display());
            };

            let Some(series) = series
                .content
                .iter()
                .find(|v| v.url == series_path.to_string_lossy())
            else {
                return error!("Failed to find series ({})", series_path.display());
            };
            debug!("Found series: {series:?}");

            if let Err(e) = komf_manager.identify(&series.library_id, &series.id).await {
                error!(
                    "Failed to identify series ({}) with error: {e}",
                    series_path.display()
                );
            }

            let _ = status_tx.send(format!("Identified series ({})", series_path.display()));
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
        if self.input_mode == InputMode::Editing && self.current_tab == Tab::Metadata {
            self.input_mode = InputMode::Normal;
        } else {
            self.current_tab = Tab::ChaptersList;
        }
    }

    /// Refreshes the chapter list
    pub fn handle_refresh(&mut self) {
        let series_path = self.get_current_series().path;

        if let Some(series) = self
            .series_list
            .items_state
            .iter_mut()
            .find(|v| v.path == series_path)
        {
            let Ok(mut new_chapters) = crate::data::get_cbz_list(&series_path) else {
                error!(
                    "Failed to get cbz list for series ({})",
                    series_path.display()
                );
                return;
            };

            new_chapters.sort();
            series.chapters = ChapterList::from_iter(new_chapters);
            self.series_list.items = self.series_list.items_state.clone();

            let _ = self
                .status_tx
                .send("Refreshed chapters list in series".to_string());
        }
    }
}

impl App {
    pub fn next_tab(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.current_tab = Tab::ChaptersList,
            Tab::ChaptersList => self.current_tab = Tab::Metadata,
            Tab::Metadata | Tab::Search => {}
        }
    }

    pub fn previous_tab(&mut self) {
        match self.current_tab {
            Tab::SeriesList => self.current_tab = Tab::ChaptersList,
            Tab::ChaptersList => self.current_tab = Tab::SeriesList,
            Tab::Metadata | Tab::Search => {}
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
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
            Tab::Metadata | Tab::Search => {}
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
            Tab::Metadata | Tab::Search => {}
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
            Tab::Metadata | Tab::Search => {}
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
            Tab::Metadata | Tab::Search => {}
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
            Tab::Metadata | Tab::Search => {}
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
            Tab::Metadata | Tab::Search => {}
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
