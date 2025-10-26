//! CBZ files manager

#[macro_use]
extern crate log;

use std::fs::File;

use log::LevelFilter;
use simplelog::{WriteLogger, format_description};

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
    let log_file = File::create(Config::get_log_path()?)?;

    WriteLogger::init(
        LevelFilter::Info,
        simplelog::ConfigBuilder::new()
            .set_thread_level(LevelFilter::Error)
            .set_time_format_custom(format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .set_time_offset_to_local()
            .map_err(|e| anyhow::anyhow!("{e:?}"))?
            .build(),
        log_file,
    )?;

    let series = get_series_list(&config.manga_dir)?;

    let terminal = ratatui::init();
    let app_result = App::new(series, &config)?.run(terminal);
    ratatui::restore();
    app_result
}
