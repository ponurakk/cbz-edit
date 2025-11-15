use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::ui::{App, Tab, app::SELECTED_YELLOW};

impl App {
    pub fn render_search(&mut self, area: Rect, f: &mut Frame) {
        let mut title = Span::raw("Search");
        if self.current_tab == Tab::Search {
            title = title.style(SELECTED_YELLOW).underlined();
        }
        let title = Line::from(vec![
            Span::raw(" "),
            title,
            Span::raw(" "),
            Span::raw(format!(
                "({}/{})",
                self.series_list.found.0 + 1,
                self.series_list.found.1.len()
            )),
        ])
        .left_aligned();

        let block = Block::default()
            .title(title)
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let Some(input) = &mut self.series_list.search_text else {
            error!("Failed to get search text");
            return;
        };

        let width = area.width.max(3) - 3;
        let scroll = input.visual_scroll(width as usize);
        #[allow(clippy::cast_possible_truncation)]
        let widget = Paragraph::new(input.value())
            .scroll((0, scroll as u16))
            .block(block);

        f.render_widget(widget, area);
    }
}
