use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{
        Color, Modifier, Style,
        palette::tailwind::{CYAN, NEUTRAL},
    },
    widgets::{Scrollbar, ScrollbarOrientation},
};

use crate::ui::{App, Tab};

pub const SELECTED_STYLE: Style = Style::new()
    .fg(CYAN.c600)
    .bg(NEUTRAL.c900)
    .add_modifier(Modifier::BOLD);
pub const SELECTED_YELLOW: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);

pub const SCROLLBAR: Scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
    .begin_symbol(None)
    .track_symbol(None)
    .end_symbol(None);

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        let [series_area, chapters_area, data_area] =
            if self.current_tab == Tab::ChaptersList || self.current_tab == Tab::Metadata {
                Layout::horizontal([
                    Constraint::Percentage(20),
                    Constraint::Percentage(40),
                    Constraint::Fill(1),
                ])
                .areas(main_area)
            } else {
                Layout::horizontal([
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Fill(1),
                ])
                .areas(main_area)
            };

        let [data_info_area, data_input_area] =
            Layout::vertical([Constraint::Percentage(45), Constraint::Fill(1)]).areas(data_area);

        App::render_header(header_area, frame);
        self.render_footer(footer_area, frame);

        self.render_series(series_area, frame);
        self.render_chapters(chapters_area, frame);
        self.render_data_input(data_input_area, frame);

        self.render_info(data_info_area, frame);

        if self.show_help {
            App::render_help(main_area, frame);
        }
    }
}
