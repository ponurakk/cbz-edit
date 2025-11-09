use serde::{Deserialize, Deserializer, Serialize};

use crate::{comic_info::ComicInfo, komga::series::KomgaSeries, serializers::empty_string_as_none};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaBooksMetadata {
    pub title: String,
    pub summary: Option<String>,
    pub number: f32,
    pub tags: Vec<String>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub translator: Option<String>,
    pub year: Option<u16>,
    pub month: Option<u16>,
    pub day: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct RawBook {
    title: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    summary: Option<String>,
    #[serde(default, rename = "numberSort")]
    number: f32,
    #[serde(default)]
    authors: Vec<super::RawAuthor>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default, rename = "releaseDate")]
    release_date: Option<String>,
}

impl<'de> Deserialize<'de> for KomgaBooksMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawBook::deserialize(deserializer)?;

        let mut writer = None;
        let mut penciller = None;
        let mut translator = None;

        for author in raw.authors {
            match author.role.as_str() {
                "writer" if writer.is_none() => writer = Some(author.name),
                "penciller" if penciller.is_none() => penciller = Some(author.name),
                "translator" if translator.is_none() => translator = Some(author.name),
                _ => {}
            }
        }

        let (year, month, day) = if let Some(release_date) = raw.release_date {
            let mut parts = release_date.splitn(3, '-');
            (
                parts.next().and_then(|v| v.parse().ok()),
                parts.next().and_then(|v| v.parse().ok()),
                parts.next().and_then(|v| v.parse().ok()),
            )
        } else {
            (None, None, None)
        };

        Ok(KomgaBooksMetadata {
            title: raw.title,
            summary: raw.summary,
            number: raw.number,
            tags: raw.tags,
            writer,
            penciller,
            translator,
            year,
            month,
            day,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaBooksMedia {
    pub pages_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaBook {
    pub id: String,
    pub series_id: String,
    pub series_title: String,
    pub library_id: String,
    pub name: String,
    pub url: String,
    pub number: u32,
    pub oneshot: bool,
    pub metadata: KomgaBooksMetadata,
    pub media: KomgaBooksMedia,
}

impl KomgaBook {
    pub fn to_comic_info(&self, series: &KomgaSeries, comic_info: &ComicInfo) -> ComicInfo {
        let genre = if series.metadata.genres.is_empty() {
            None
        } else {
            Some(series.metadata.genres.join(","))
        };

        let tags = if series.metadata.tags.is_empty() {
            None
        } else {
            Some(series.metadata.tags.join(","))
        };

        ComicInfo {
            title: self.metadata.title.clone(),
            series: self.series_title.clone(),
            number: Some(self.metadata.number),
            volume: comic_info.volume,
            summary: series
                .metadata
                .summary
                .as_ref()
                .or(comic_info.summary.as_ref())
                .cloned(),
            year: self.metadata.year.or(comic_info.year),
            month: self.metadata.month.or(comic_info.month),
            day: self.metadata.day.or(comic_info.day),
            writer: self
                .metadata
                .writer
                .as_ref()
                .or(comic_info.writer.as_ref())
                .cloned(),
            penciller: self
                .metadata
                .penciller
                .as_ref()
                .or(comic_info.penciller.as_ref())
                .cloned(),
            translator: self
                .metadata
                .translator
                .as_ref()
                .or(comic_info.translator.as_ref())
                .cloned(),
            publisher: series.metadata.publisher.clone(),
            genre,
            tags,
            web: comic_info.web.clone(),
            page_count: Some(self.media.pages_count),
            language_iso: series.metadata.language.clone(),
            manga: series
                .metadata
                .reading_direction
                .clone()
                .map(Into::into)
                .unwrap_or_default(),
            age_rating: series
                .metadata
                .age_rating
                .map(Into::into)
                .unwrap_or_default(),
            count: series.metadata.total_book_count.or(comic_info.count),
        }
    }
}
