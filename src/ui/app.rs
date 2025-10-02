use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::tailwind::NEUTRAL},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget, Widget,
    },
};
use ratatui_image::StatefulImage;

use crate::ui::{App, Tab};

const SELECTED_STYLE: Style = Style::new().bg(NEUTRAL.c900).add_modifier(Modifier::BOLD);

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [series_area, chapters_area, data_area] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Fill(1),
        ])
        .areas(main_area);

        let [data_info_area, data_input_area] =
            Layout::vertical([Constraint::Percentage(30), Constraint::Fill(1)]).areas(data_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);

        self.render_series(series_area, buf);
        self.render_chapters(chapters_area, buf);
        self.render_data_input(data_input_area, buf);
        self.render_info(data_info_area, buf);
    }
}

impl App {
    pub fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("CBZ file manager")
            .bold()
            .centered()
            .render(area, buf);
    }

    pub fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    pub fn render_series(&mut self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new()
            .title(Line::raw("Series").left_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);
        if self.current_tab == Tab::SeriesList {
            block = block.title(Line::raw("*").left_aligned());
        }

        let items: Vec<ListItem> = self.series_list.items.iter().map(ListItem::from).collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.series_list.state);
    }

    pub fn render_chapters(&mut self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new()
            .title(Line::raw("Chapters").left_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);
        if self.current_tab == Tab::ChaptersList {
            block = block.title(Span::raw("*"));
        }

        let Some(series) = self
            .series_list
            .items_state
            .get_mut(self.series_list.state.selected().unwrap_or(0))
        else {
            return;
        };

        let items: Vec<ListItem> = series.chapters.items.iter().map(ListItem::from).collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut series.chapters.state);
    }

    pub fn render_info(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Info").left_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let inner_area = block.inner(area);
        Widget::render(block, area, buf);

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(inner_area);

        StatefulImage::new().render(areas[1], buf, &mut self.image);
    }

    pub fn render_data_input(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .comic
            .fields
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let input = &self.comic.field_inputs[i];
                let line = format!("{name:<12}: {input}");
                ListItem::new(Line::raw(line))
            })
            .collect();

        let mut block = Block::new()
            .title("Edit Metadata")
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        if self.current_tab == Tab::Metadata {
            block = block.title(Span::raw("*"));
        }

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, &mut self.comic.fields_state);
    }
}
