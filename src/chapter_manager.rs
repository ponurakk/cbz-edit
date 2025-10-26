use std::{path::PathBuf, time::Instant};

use futures::{StreamExt, stream};
use tokio::sync::watch;

use crate::{
    comic_info::ComicInfo,
    ui::list::Chapter,
    zip_util::{derive_comic_info, modify_comic_info, replace_comic_info, volume_comic_info},
};

fn get_title(chapter: &Chapter) -> String {
    let path = chapter.path.clone();
    chapter
        .title
        .clone()
        .unwrap_or_else(|| path.display().to_string())
}

async fn process_chapter_info<F>(
    chapter: Chapter,
    info: ComicInfo,
    status_tx: watch::Sender<String>,
    i: usize,
    chapters_len: usize,
    process_fn: F,
) -> anyhow::Result<()>
where
    F: FnOnce(&PathBuf, &ComicInfo) -> anyhow::Result<()> + std::marker::Send + 'static,
{
    let title = get_title(&chapter);
    let _ = status_tx.send(format!("Processing {}/{}: {}", i + 1, chapters_len, title));

    tokio::task::spawn_blocking(move || process_fn(&chapter.path, &info)).await??;
    Ok(())
}

/// Save the inputs to the [`ComicInfo`]
pub async fn save_chapter_info(
    chapter: Chapter,
    info: ComicInfo,
    status_tx: watch::Sender<String>,
) -> anyhow::Result<()> {
    let total_start = Instant::now();
    process_chapter_info(chapter, info, status_tx.clone(), 0, 1, replace_comic_info).await?;

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
    process_chapter_info(chapter, info, status_tx, i, chapters_len, modify_comic_info).await
}

/// Save the inputs to the [`ComicInfo`]
async fn update_derived(
    chapter: Chapter,
    info: ComicInfo,
    status_tx: watch::Sender<String>,
    i: usize,
    chapters_len: usize,
) -> anyhow::Result<()> {
    process_chapter_info(chapter, info, status_tx, i, chapters_len, derive_comic_info).await
}

/// Save the inputs to the [`ComicInfo`]
async fn update_volume(
    chapter: Chapter,
    info: ComicInfo,
    status_tx: watch::Sender<String>,
    i: usize,
    chapters_len: usize,
) -> anyhow::Result<()> {
    process_chapter_info(chapter, info, status_tx, i, chapters_len, volume_comic_info).await
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

/// Updates derived info
pub async fn update_chapter_numbering(
    chapters: Vec<Chapter>,
    status_tx: watch::Sender<String>,
) -> anyhow::Result<()> {
    let chapters_len = chapters.len();
    // TODO: Make this in config
    let concurrency_limit = num_cpus::get();
    let total_start = Instant::now();

    stream::iter(chapters.into_iter().enumerate())
        .map(|(i, chapter)| {
            let status_tx = status_tx.clone();
            let mut info = ComicInfo {
                volume: chapter.volume,
                number: chapter.chapter,
                translator: Some(chapter.translators.join(",")),
                ..Default::default()
            };
            if let Some(title) = &chapter.title {
                info.title.clone_from(title);
            }

            update_derived(chapter, info, status_tx, i, chapters_len)
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

pub async fn update_volume_numbering(
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
            update_volume(chapter, info, status_tx, i, chapters_len)
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
