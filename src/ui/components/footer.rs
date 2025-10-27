use ratatui::{Frame, layout::Rect, widgets::Paragraph};

use crate::ui::App;

impl App {
    pub fn render_footer(&self, area: Rect, f: &mut Frame) {
        let status = self.status_rx.borrow().clone();
        let footer = Paragraph::new(status).left_aligned();
        f.render_widget(footer, area);
    }
}
