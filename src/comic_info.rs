//! The `ComicInfo` struct used in CBZ files

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Default, Serialize, Clone, Copy)]
pub enum ComicInfoManga {
    #[default]
    Unknown,
    Yes,
    No,
    YesAndRightToLeft,
}

impl FromStr for ComicInfoManga {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Yes" => Ok(Self::Yes),
            "No" => Ok(Self::No),
            "YesAndRightToLeft" => Ok(Self::YesAndRightToLeft),
            _ => Ok(Self::Unknown),
        }
    }
}

impl Display for ComicInfoManga {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => write!(f, "Yes"),
            Self::No => write!(f, "No"),
            Self::YesAndRightToLeft => write!(f, "YesAndRightToLeft"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl<'de> Deserialize<'de> for ComicInfoManga {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        match s.as_str() {
            "Yes" => Ok(Self::Yes),
            "No" => Ok(Self::No),
            "YesAndRightToLeft" => Ok(Self::YesAndRightToLeft),
            _ => Ok(Self::Unknown),
        }
    }
}

#[derive(Debug, Default, Serialize, Clone, Copy)]
pub enum ComicInfoAgeRating {
    #[default]
    Unknown,
    /// Kodomo
    Everyone,
    /// Shonen / Shojo
    Teen,
    /// Seinen / Josei
    #[serde(rename = "Mature 17+")]
    Mature17Plus,
    /// Hentai / Erotic
    #[serde(rename = "Adults Only 18+")]
    AdultsOnly18Plus,
}

impl FromStr for ComicInfoAgeRating {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Everyone" => Ok(Self::Everyone),
            "Teen" => Ok(Self::Teen),
            "Mature 17+" => Ok(Self::Mature17Plus),
            "Adults Only 18+" => Ok(Self::AdultsOnly18Plus),
            _ => Ok(Self::Unknown),
        }
    }
}

impl From<&str> for ComicInfoAgeRating {
    fn from(value: &str) -> Self {
        match value {
            "Everyone" => Self::Everyone,
            "Teen" => Self::Teen,
            "Mature 17+" => Self::Mature17Plus,
            "Adults Only 18+" => Self::AdultsOnly18Plus,
            _ => Self::Unknown,
        }
    }
}

impl Display for ComicInfoAgeRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Everyone => write!(f, "Everyone"),
            Self::Teen => write!(f, "Teen"),
            Self::Mature17Plus => write!(f, "Mature 17+"),
            Self::AdultsOnly18Plus => write!(f, "Adults Only 18+"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl<'de> Deserialize<'de> for ComicInfoAgeRating {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        match s.as_str() {
            "Everyone" => Ok(Self::Everyone),
            "Teen" => Ok(Self::Teen),
            "Mature 17+" => Ok(Self::Mature17Plus),
            "Adults Only 18+" => Ok(Self::AdultsOnly18Plus),
            _ => Ok(Self::Unknown),
        }
    }
}

/// Information about a comic book
///
/// From <https://anansi-project.github.io/docs/comicinfo/documentation>
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComicInfo {
    /// Title of the book.
    pub title: String,

    /// Title of the series the book is part of.
    pub series: String,

    /// Number of the book in the series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<f32>,

    /// Volume containing the book. Volume is a notion that is specific to US Comics, where the
    /// same series can have multiple volumes. Volumes can be referenced by number (1, 2, 3…) or by
    /// year (2018, 2020…).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u32>,

    /// A description or summary of the book.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Release year of the book.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,

    /// Release month of the book.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<u16>,

    /// Release day of the book.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<u8>,

    /// Person or organization responsible for creating the scenario. (Multiple writers should be
    /// comma separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer: Option<String>,

    /// Person or organization responsible for drawing the art. (Multiple pencillers should be
    /// comma separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub penciller: Option<String>,

    /// A person or organization who renders a text from one language into another, or from an
    /// older form of a language into the modern form. (Multiple translators should be comma
    /// separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translator: Option<String>,

    /// A person or organization responsible for publishing, releasing, or issuing a resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Genre of the book or series. For example, Science-Fiction or Shonen.
    /// It is accepted that multiple values are comma separated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,

    /// Tags of the book or series. For example, ninja or school life.
    /// It is accepted that multiple values are comma separated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// A URL pointing to a reference website for the book.
    /// It is accepted that multiple values are space separated. If a space is a part of the url it
    /// must be [percent encoded](https://datatracker.ietf.org/doc/html/rfc2396#section-2.4.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<String>,

    /// The number of pages in the book.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<u32>,

    /// A language code describing the language of the book.
    #[serde(rename = "LanguageISO", skip_serializing_if = "Option::is_none")]
    pub language_iso: Option<String>,

    /// Whether the book is a manga. This also defines the reading direction as right-to-left when set to `YesAndRightToLeft`.
    #[serde(default)]
    pub manga: ComicInfoManga,

    /// The age rating of the book.
    #[serde(default)]
    pub age_rating: ComicInfoAgeRating,

    /// The total number of books in the series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}

impl ComicInfo {
    pub fn new(title: String) -> Self {
        Self {
            title: title.clone(),
            series: title,
            ..Default::default()
        }
    }

    /// Updates fields that are the same across all chapters in the series
    pub fn update_shared_fields(&mut self, comic_info: &Self) {
        self.series.clone_from(&comic_info.series);
        self.summary.clone_from(&comic_info.summary);
        self.writer.clone_from(&comic_info.writer);
        self.penciller.clone_from(&comic_info.penciller);
        self.publisher.clone_from(&comic_info.publisher);
        self.genre.clone_from(&comic_info.genre);
        self.tags.clone_from(&comic_info.tags);
        self.web.clone_from(&comic_info.web);
        self.language_iso.clone_from(&comic_info.language_iso);
        self.manga = comic_info.manga;
        self.age_rating = comic_info.age_rating;
        self.count = comic_info.count;
    }

    /// Updates fields that can be derived from filename
    pub fn update_derived_fields(&mut self, comic_info: &Self) {
        self.title.clone_from(&comic_info.title);
        self.translator.clone_from(&comic_info.translator);
        self.number = comic_info.number;
        self.volume = comic_info.volume;
    }

    /// Updates the volume number
    pub fn update_volume(&mut self, comic_info: &Self) {
        self.volume = comic_info.volume;
    }
}
