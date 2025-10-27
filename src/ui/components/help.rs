use ratatui::{Frame, layout::Rect};

use crate::ui::{App, widgets::help_popup::HelpPopup};

impl App {
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
