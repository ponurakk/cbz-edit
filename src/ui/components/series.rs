use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::Stylize,
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem},
};

use crate::ui::{
    App, Tab,
    app::{SCROLLBAR, SELECTED_STYLE, SELECTED_YELLOW},
};

impl App {
    pub fn render_series(&mut self, area: Rect, f: &mut Frame) {
        let mut title = Span::raw("Series");
        if self.current_tab == Tab::SeriesList {
            title = title.style(SELECTED_YELLOW).underlined();
        }
        let title = Line::from(vec![Span::raw(" "), title, Span::raw(" ")]).left_aligned();

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let items: Vec<ListItem> = self.series_list.items.iter().map(ListItem::from).collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        let inner = area.inner(Margin::new(0, 1));
        f.render_stateful_widget(list, area, &mut self.series_list.state);
        f.render_stateful_widget(SCROLLBAR, inner, &mut self.series_list.scroll_state);
    }
}
