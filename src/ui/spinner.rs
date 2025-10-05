use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

/// Spinner frames
const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Debug, Default)]
pub struct SpinnerState {
    pub tick_count: usize,
}

impl SpinnerState {
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }
}

/// A simple animated spinner widget uwu
#[derive(Debug, Default)]
pub struct Spinner<'a> {
    title: &'a str,
}

impl<'a> Spinner<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { title: label }
    }
}

impl StatefulWidget for Spinner<'_> {
    type State = SpinnerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let spinner = FRAMES[state.tick_count % FRAMES.len()];

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
