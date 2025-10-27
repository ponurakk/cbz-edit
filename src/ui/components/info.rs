use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use ratatui_image::{Resize, ResizeEncodeRender, StatefulImage};

use crate::{
    managers::image::ImagesState,
    ui::{App, widgets::spinner::Spinner},
};

impl App {
    pub fn render_info(&mut self, area: Rect, f: &mut Frame) {
        let ImagesState::Ready(ref mut images) = self.image_manager.images else {
            f.render_stateful_widget(
                Spinner::new(" Pages "),
                area,
                &mut self.image_manager.spinner,
            );
            return;
        };

        let block = Block::new()
            .title(Line::raw(" Pages ").left_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(inner_area);

        if let Some(prev_index) = self.image_manager.current.checked_sub(1)
            && let Some(img) = images.get_mut(prev_index)
        {
            if let Some(rect) = img.needs_resize(
                &Resize::Fit(Some(ratatui_image::FilterType::Nearest)),
                areas[0],
            ) {
                img.resize_encode(&Resize::Fit(Some(ratatui_image::FilterType::Nearest)), rect);
            }
            f.render_stateful_widget(StatefulImage::default(), areas[0], img);
        }

        if let Some(img) = images.get_mut(self.image_manager.current) {
            let middle_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
                .split(areas[1]);

            f.render_stateful_widget(StatefulImage::default(), middle_split[0], img);

            let label = Paragraph::new(Span::styled(
                "★ Selected ★",
                Style::default().fg(Color::Cyan),
            ))
            .alignment(Alignment::Center);

            f.render_widget(label, middle_split[1]);
        }

        if let Some(img) = images.get_mut(self.image_manager.current + 1) {
            f.render_stateful_widget(StatefulImage::default(), areas[2], img);
        }
    }
}
