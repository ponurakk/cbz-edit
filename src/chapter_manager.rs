use std::time::Instant;

use futures::{StreamExt, stream};
use tokio::sync::watch;

use crate::{
    comic_info::ComicInfo,
    ui::list::Chapter,
    zip_util::{modify_comic_info, replace_comic_info},
};

/// Save the inputs to the [`ComicInfo`]
pub async fn save_chapter_info(
    chapter: Chapter,
    comic_info: ComicInfo,
    status_tx: watch::Sender<String>,
) -> anyhow::Result<()> {
    let total_start = Instant::now();
    let path = chapter.path.clone();
    let title = chapter
        .title
        .clone()
        .unwrap_or_else(|| path.display().to_string());
    let _ = status_tx.send(format!("Processing: {title}"));

    tokio::task::spawn_blocking(move || replace_comic_info(&chapter.path, &comic_info)).await??;

    let total_duration = total_start.elapsed();
    let _ = status_tx.send(format!(
        "All done~ processed chapter in {total_duration:.2?} 🎉"
    ));
    Ok(())
}

/// Save the inputs to the [`ComicInfo`]
async fn update_info(
    chapter: Chapter,
    info: ComicInfo,
    status_tx: watch::Sender<String>,
    i: usize,
    chapters_len: usize,
) -> anyhow::Result<()> {
    let path = chapter.path.clone();
    let title = chapter
        .title
        .clone()
        .unwrap_or_else(|| path.display().to_string());

    let _ = status_tx.send(format!("Processing {}/{}: {}", i + 1, chapters_len, title));

    tokio::task::spawn_blocking(move || modify_comic_info(&path, &info)).await??;

    Ok(())
}

/// Save the inputs to the [`ComicInfo`]
pub async fn save_series_info(
    chapters: Vec<Chapter>,
    comic_info: ComicInfo,
    status_tx: watch::Sender<String>,
) -> anyhow::Result<()> {
    let chapters_len = chapters.len();
    // TODO: Make this in config
    let concurrency_limit = num_cpus::get();
    let total_start = Instant::now();

    stream::iter(chapters.into_iter().enumerate())
        .map(|(i, chapter)| {
            let status_tx = status_tx.clone();
            let info = comic_info.clone();
            update_info(chapter, info, status_tx, i, chapters_len)
        })
        .buffer_unordered(concurrency_limit)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    let total_duration = total_start.elapsed();

    let _ = status_tx.send(format!(
        "All done~ processed {chapters_len} chapters in {total_duration:.2?} 🎉"
    ));

    Ok(())
}
