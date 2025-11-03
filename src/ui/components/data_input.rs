use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::{self, border::PLAIN},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use tui_input::Input;

use crate::{
    managers::comic_form::ComicFormState,
    ui::{App, InputMode, Tab, app::SELECTED_YELLOW, widgets::spinner::Spinner},
};

impl App {
    pub fn render_data_input(&mut self, area: Rect, f: &mut Frame) {
        let ComicFormState::Ready(ref comic) = self.comic_manager.comic else {
            f.render_stateful_widget(
                Spinner::new(" Edit Metadata "),
                area,
                &mut self.comic_manager.spinner,
            );
            return;
        };

        let mut title = Span::raw("Edit Metadata");
        if self.current_tab == Tab::Metadata {
            title = title.style(SELECTED_YELLOW).underlined();
        }
        let title = Line::from(vec![Span::raw(" "), title, Span::raw(" ")]).left_aligned();

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);
        let inner = block.inner(area);
        f.render_widget(block, area);

        // Split screen into two columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .flex(ratatui::layout::Flex::SpaceAround)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(45)])
            .split(inner);

        // split the fields into two halves
        let mid = comic.fields.len().div_ceil(2); // left gets the extra if odd
        let (left_fields, right_fields) = comic.fields.split_at(mid);

        // Left column (vertical split for each field)
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3); left_fields.len()])
            .split(columns[0]);

        for (i, (label, input)) in left_fields.iter().enumerate() {
            let global_index = i; // real index from form.fields
            self.render_field(
                f,
                label,
                input,
                global_index,
                comic.active_index,
                left_chunks[i],
            );
        }

        // Right column (vertical split for each field)
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3); right_fields.len()])
            .split(columns[1]);

        for (i, (label, input)) in right_fields.iter().enumerate() {
            let global_index = mid + i; // real index continues after left column
            self.render_field(
                f,
                label,
                input,
                global_index,
                comic.active_index,
                right_chunks[i],
            );
        }
    }

    // helper to render a single field block
    fn render_field(
        &self,
        f: &mut Frame,
        label: &str,
        input: &Input,
        idx: usize,
        active_index: usize,
        area: ratatui::layout::Rect,
    ) {
        let title = Line::raw(label).bold().left_aligned();
        let mut block = Block::default()
            .title(title)
            .padding(Padding::horizontal(1))
            .borders(Borders::NONE);

        block = if idx == active_index {
            if self.input_mode == InputMode::Editing && self.current_tab == Tab::Metadata {
                block.border_style(Style::default().fg(Color::Red))
            } else {
                block.border_style(Style::default().fg(Color::Cyan))
            }
        } else {
            block
        };

        let width = area.width.max(3) - 3;
        let scroll = input.visual_scroll(width as usize);
        #[allow(clippy::cast_possible_truncation)]
        let widget = Paragraph::new(input.value())
            .scroll((0, scroll as u16))
            .block(block);

        f.render_widget(widget, area);

        let bottom_border = format!(
            "{}{}{}",
            PLAIN.bottom_left,
            PLAIN
                .horizontal_bottom
                .repeat(area.width.saturating_sub(2) as usize),
            PLAIN.bottom_right
        );

        let border_paragraph =
            Paragraph::new(bottom_border).style(Style::default().fg(if idx == active_index {
                if self.input_mode == InputMode::Editing && self.current_tab == Tab::Metadata {
                    Color::Red
                } else {
                    Color::Cyan
                }
            } else {
                Color::White
            }));

        let border_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        f.render_widget(border_paragraph, border_area);

        // Cursor positioning
        #[allow(clippy::cast_possible_truncation)]
        if idx == active_index
            && self.input_mode == InputMode::Editing
            && self.current_tab == Tab::Metadata
        {
            let x = if input.cursor() >= (area.width - 2).into() {
                area.x + area.width - 2
            } else {
                area.x + input.cursor() as u16 + 1 // +1 because left border
            };
            let y = area.y + 1; // below top border
            f.set_cursor_position((x, y));
        }
    }
}
