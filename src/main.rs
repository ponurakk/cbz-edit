//! CBZ files manager

use crate::{data::get_series_list, ui::App};

mod comic_info;
mod data;
mod ui;

fn main() -> anyhow::Result<()> {
    let series = get_series_list("/run/media/ponurakk/Manga")?;

    let terminal = ratatui::init();
    let app_result = App::new(series).run(terminal);
    ratatui::restore();
    app_result
}
