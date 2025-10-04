use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

/// Spinner frames
const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A simple animated spinner widget uwu
pub struct Spinner<'a> {
    title: &'a str,
    frame: usize,
}

impl<'a> Spinner<'a> {
    pub fn new(label: &'a str, frame: usize) -> Self {
        Self {
            title: label,
            frame,
        }
    }
}

impl Widget for Spinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let spinner = FRAMES[self.frame % FRAMES.len()];

        let text = Line::from(vec![
            Span::styled(spinner.to_string(), Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::raw("Loading…"),
        ]);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title)
            .border_set(symbols::border::ROUNDED);

        let inner = block.inner(area);
        block.render(area, buf);
        text.render(inner, buf);
    }
}
