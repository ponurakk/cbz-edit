use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{
        Color, Modifier, Style, Stylize,
        palette::tailwind::{CYAN, NEUTRAL},
    },
    symbols::{self, border::PLAIN},
    text::{Line, Span},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation,
    },
};
use ratatui_image::{Resize, ResizeEncodeRender, StatefulImage};
use tui_input::Input;

use crate::ui::{
    App, ComicFormState, InputMode, Tab, image::ImagesState, popup::HelpPopup, spinner::Spinner,
};

const SELECTED_STYLE: Style = Style::new()
    .fg(CYAN.c600)
    .bg(NEUTRAL.c900)
    .add_modifier(Modifier::BOLD);
const SELECTED_YELLOW: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);

const SCROLLBAR: Scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
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

impl App {
    pub fn render_header(area: Rect, f: &mut Frame) {
        let title = Paragraph::new("CBZ file manager").bold().centered();
        f.render_widget(title, area);
    }

    pub fn render_footer(&self, area: Rect, f: &mut Frame) {
        let status = self.status_rx.borrow().clone();
        let footer = Paragraph::new(status).left_aligned();
        f.render_widget(footer, area);
    }

    pub fn render_series(&mut self, area: Rect, f: &mut Frame) {
        let mut title = Span::raw("Series");
        if self.current_tab == Tab::SeriesList {
            title = title.style(SELECTED_YELLOW).underlined();
        }
        let title = Line::from(vec![Span::raw(" "), title, Span::raw(" ")]).left_aligned();

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
        let mut title = Span::raw("Chapters");
        if self.current_tab == Tab::ChaptersList {
            title = title.style(SELECTED_YELLOW).underlined();
        }
        let mut title = Line::from(vec![Span::raw(" "), title, Span::raw(" ")]).left_aligned();

        let Some(series) = self
            .series_list
            .items_state
            .get_mut(self.series_list.state.selected().unwrap_or(0))
        else {
            return;
        };

        let selected_count = if series.chapters.selected.is_empty() {
            String::new()
        } else {
            format!("{}/", series.chapters.selected.len())
        };

        title.push_span(Span::raw(format!(
            "({}{}) ",
            selected_count,
            series.chapters.items_state.len(),
        )));

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let items: Vec<ListItem> = series
            .chapters
            .items
            .iter()
            .enumerate()
            .map(|(i, chapter)| {
                let mut item =
                    ListItem::new(chapter.get_title(series.chapters.selected.contains(&i)));

                if series.chapters.selected.contains(&i) {
                    item = item.style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    );
                }

                item
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        let inner = area.inner(Margin::new(0, 1));
        f.render_stateful_widget(list, area, &mut series.chapters.state);
        f.render_stateful_widget(SCROLLBAR, inner, &mut series.chapters.scroll_state);
    }

    pub fn render_info(&mut self, area: Rect, f: &mut Frame) {
        let ImagesState::Ready(ref mut images) = self.image_manager.images else {
            f.render_stateful_widget(
                Spinner::new(" Info "),
                area,
                &mut self.image_manager.spinner,
            );
            return;
        };

        let block = Block::new()
            .title(Line::raw(" Info ").left_aligned())
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
            if self.input_mode == InputMode::Editing {
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
                if self.input_mode == InputMode::Editing {
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
        if idx == active_index && self.input_mode == InputMode::Editing {
            let x = if input.cursor() >= (area.width - 2).into() {
                area.x + area.width - 2
            } else {
                area.x + input.cursor() as u16 + 1 // +1 because left border
            };
            let y = area.y + 1; // below top border
            f.set_cursor_position((x, y));
        }
    }

    pub fn render_help(area: Rect, f: &mut Frame) {
        let popup = HelpPopup::default().lines(vec![
            ("k/↑", "Go Up"),
            ("j/↓", "Go Down"),
            ("h/←", "Change pane to left"),
            ("l/→", "Change pane to right"),
            ("g", "Go to top"),
            ("G", "Go to bottom"),
            ("<space>", "Toggle selection"),
            ("?", "Toggle help"),
            ("Ctrl+c", "Close"),
            ("Ctrl+f", "Save chapter numberings"),
            ("Ctrl+s", "Save chapter info"),
            ("Ctrl+d", "Save series info"),
        ]);

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };

        f.render_widget(popup, popup_area);
    }
}
