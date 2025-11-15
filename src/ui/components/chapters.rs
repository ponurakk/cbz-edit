use std::collections::{HashMap, HashSet};

use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem},
};

use crate::ui::{
    App, Tab,
    app::{SCROLLBAR, SELECTED_STYLE, SELECTED_YELLOW},
};

impl App {
    pub fn render_chapters(&mut self, area: Rect, f: &mut Frame) {
        let mut title = Span::raw("Chapters");
        if self.current_tab == Tab::ChaptersList {
            title = title.style(SELECTED_YELLOW).underlined();
        }
        let mut title = Line::from(vec![Span::raw(" "), title, Span::raw(" ")]).left_aligned();

        let Some(series) = self
            .series_list
            .items_state
            .get_mut(self.series_list.state.selected().unwrap_or(0))
        else {
            return;
        };

        let selected_count = if series.chapters.selected.is_empty() {
            String::new()
        } else {
            format!("{}/", series.chapters.selected.len())
        };

        title.push_span(Span::raw(format!(
            "({}{}) ",
            selected_count,
            series.chapters.items_state.len(),
        )));

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let mut counts: HashMap<Option<u32>, usize> = HashMap::new();

        if series.name != self.config.komga.oneshots_dir {
            for c in &series.chapters.items {
                let key = c.chapter.map(f32::to_bits);
                *counts.entry(key).or_insert(0) += 1;
            }
        }

        let duplicates: HashSet<Option<u32>> = counts
            .into_iter()
            .filter(|(_, c)| *c > 1)
            .map(|(k, _)| k)
            .collect();

        let items: Vec<ListItem> = series
            .chapters
            .items
            .iter()
            .enumerate()
            .map(|(i, chapter)| {
                let mut item =
                    ListItem::new(chapter.get_title(series.chapters.selected.contains(&i)));

                if series.chapters.selected.contains(&i) {
                    item = item.style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    );
                }

                let key = chapter.chapter.map(f32::to_bits);
                if duplicates.contains(&key) {
                    item = item.style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                }

                item
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        let inner = area.inner(Margin::new(0, 1));
        f.render_stateful_widget(list, area, &mut series.chapters.state);
        f.render_stateful_widget(SCROLLBAR, inner, &mut series.chapters.scroll_state);
    }
}
