use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaBooksMetadata {
    pub title: String,
    pub summary: String,
    pub number: f32,
    pub tags: Vec<String>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub translator: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawBook {
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default, rename = "numberSort")]
    number: f32,
    #[serde(default)]
    authors: Vec<super::RawAuthor>,
    #[serde(default)]
    tags: Vec<String>,
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
            tags: raw.tags,
            writer,
            penciller,
            translator,
        })
    }
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
}
