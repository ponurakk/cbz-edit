use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize, palette::tailwind::NEUTRAL},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation,
    },
};
use ratatui_image::StatefulImage;

use crate::ui::{App, Tab};

const SELECTED_STYLE: Style = Style::new().bg(NEUTRAL.c900).add_modifier(Modifier::BOLD);
const SELECTED_YELLOW: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);

const SCROLLBAR: Scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
    .begin_symbol(None)
    .track_symbol(None)
    .end_symbol(None);

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [series_area, chapters_area, data_area] = if self.current_tab == Tab::SeriesList {
            Layout::horizontal([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Fill(1),
            ])
            .areas(main_area)
        } else {
            Layout::horizontal([
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Fill(1),
            ])
            .areas(main_area)
        };

        let [data_info_area, data_input_area] =
            Layout::vertical([Constraint::Percentage(30), Constraint::Fill(1)]).areas(data_area);

        App::render_header(header_area, frame);
        App::render_footer(footer_area, frame);

        self.render_series(series_area, frame);
        self.render_chapters(chapters_area, frame);
        self.render_data_input(data_input_area, frame);
        self.render_info(data_info_area, frame);
    }
}

impl App {
    pub fn render_header(area: Rect, f: &mut Frame) {
        let title = Paragraph::new("CBZ file manager").bold().centered();
        f.render_widget(title, area);
    }

    pub fn render_footer(area: Rect, f: &mut Frame) {
        let footer =
            Paragraph::new("Use ↓↑ to move, ←→ to change tabs, g/G to go top/bottom.").centered();
        f.render_widget(footer, area);
    }

    pub fn render_series(&mut self, area: Rect, f: &mut Frame) {
        let mut title = Line::raw("Series").left_aligned();
        if self.current_tab == Tab::SeriesList {
            title = title.style(SELECTED_YELLOW).underlined();
        }

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

    pub fn render_chapters(&mut self, area: Rect, f: &mut Frame) {
        let mut title = Line::raw("Chapters").left_aligned();
        if self.current_tab == Tab::ChaptersList {
            title = title.style(SELECTED_YELLOW).underlined();
        }

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

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

        let inner = area.inner(Margin::new(0, 1));
        f.render_stateful_widget(list, area, &mut series.chapters.state);
        f.render_stateful_widget(SCROLLBAR, inner, &mut series.chapters.scroll_state);
    }

    pub fn render_info(&mut self, area: Rect, f: &mut Frame) {
        let block = Block::new()
            .title(Line::raw("Info").left_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(inner_area);

        f.render_stateful_widget(StatefulImage::default(), areas[1], &mut self.image);
    }

    pub fn render_data_input(&mut self, area: Rect, f: &mut Frame) {
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

        f.render_stateful_widget(list, area, &mut self.comic.fields_state);
    }
}
