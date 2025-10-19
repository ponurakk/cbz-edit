//! CBZ files manager

use crate::{config::Config, data::get_series_list, ui::App};

mod chapter_manager;
mod comic_info;
mod config;
mod data;
mod komga;
mod ui;
mod zip_util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::read()?;

    let series = get_series_list(&config.manga_dir)?;

    let terminal = ratatui::init();
    let app_result = App::new(series, &config)?.run(terminal);
    ratatui::restore();
    app_result
}
