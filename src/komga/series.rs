use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaSeriesMetadata {
    pub title: String,
    pub summary: String,
    pub publisher: String,
    pub age_rating: Option<u32>,
    pub language: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub total_book_count: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaSeriesBooksMetadata {
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub translator: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawMetadata {
    authors: Vec<super::RawAuthor>,
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

#[derive(Debug, Serialize, Deserialize)]
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
