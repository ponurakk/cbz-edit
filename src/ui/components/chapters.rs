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
