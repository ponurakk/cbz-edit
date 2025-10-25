use ratatui::{
    layout::Constraint,
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Row, Table, Widget},
};

#[derive(Debug, Default)]
pub struct HelpPopup<'a> {
    title: Line<'a>,
    lines: Vec<(&'a str, &'a str)>,
}

impl Widget for HelpPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let rows: Vec<Row> = self
            .lines
            .iter()
            .map(|(left, right)| {
                Row::new(vec![
                    Cell::from(Span::from(*left).into_left_aligned_line())
                        .style(Style::default().fg(Color::Cyan)),
                    Cell::from(Span::from(*right).into_left_aligned_line())
                        .style(Style::default().fg(Color::Cyan)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            &[Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(block)
        .column_spacing(2)
        .row_highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )
        .highlight_symbol("> ");

        table.render(area, buf);
    }
}

impl<'a> HelpPopup<'a> {
    pub fn lines(mut self, text: Vec<(&'a str, &'a str)>) -> Self {
        self.lines = text;
        self
    }
}
