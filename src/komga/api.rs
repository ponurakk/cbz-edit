use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaSeriesMetadata {
    pub title: String,
    pub summary: String,
    pub publisher: String,
    pub age_rating: u32,
    pub language: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub total_book_count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaSeriesBooksMetadata {
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub translator: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawAuthor {
    name: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct RawMetadata {
    authors: Vec<RawAuthor>,
}

impl<'de> Deserialize<'de> for KomgaSeriesBooksMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawMetadata::deserialize(deserializer)?;

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

        Ok(KomgaSeriesBooksMetadata {
            writer,
            penciller,
            translator,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaSeries {
    pub id: String,
    pub library_id: String,
    pub name: String,
    pub url: String,
    pub books_count: u32,
    pub oneshot: bool,
    pub metadata: KomgaSeriesMetadata,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaBooksMetadata {
    pub title: String,
    pub summary: String,
    pub number: String,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub translator: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawBook {
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    number: String,
    #[serde(default)]
    authors: Vec<RawAuthor>,
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

        Ok(KomgaBooksMetadata {
            title: raw.title,
            summary: raw.summary,
            number: raw.number,
            writer,
            penciller,
            translator,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaBook {
    pub id: String,
    pub series_id: String,
    pub series_title: String,
    pub name: String,
    pub url: String,
    pub number: u32,
    pub oneshot: bool,
    pub metadata: KomgaBooksMetadata,
}

// Done so only allowed types can be used
pub trait KomgaItem {}
impl KomgaItem for KomgaSeries {}
impl KomgaItem for KomgaBook {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaResponse<T: KomgaItem> {
    pub total_elements: i64,
    pub total_pages: i32,
    pub content: Vec<T>,
}

pub type KomgaSeriesResponse = KomgaResponse<KomgaSeries>;
pub type KomgaBookResponse = KomgaResponse<KomgaBook>;
