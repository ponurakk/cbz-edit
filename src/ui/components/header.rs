use ratatui::{Frame, layout::Rect, style::Stylize, widgets::Paragraph};

use crate::ui::App;

impl App {
    pub fn render_header(area: Rect, f: &mut Frame) {
        let title = Paragraph::new("CBZ file manager").bold().centered();
        f.render_widget(title, area);
    }
}
