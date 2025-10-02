//! The `ComicInfo` struct used in CBZ files

use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Default, Serialize, Clone, Copy)]
pub enum ComicInfoManga {
    #[default]
    Unknown,
    Yes,
    No,
    YesAndRightToLeft,
}

impl From<String> for ComicInfoManga {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Yes" => Self::Yes,
            "No" => Self::No,
            "YesAndRightToLeft" => Self::YesAndRightToLeft,
            _ => Self::Unknown,
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
    Mature17Plus,
    /// Hentai / Erotic
    AdultsOnly18Plus,
}

impl From<String> for ComicInfoAgeRating {
    fn from(value: String) -> Self {
        match value.as_str() {
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

#[derive(Debug, Default, Serialize, Deserialize)]
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

    #[serde(default)]
    pub age_rating: ComicInfoAgeRating,
}

impl ComicInfo {
    pub fn new(title: String) -> Self {
        Self {
            title: title.clone(),
            series: title,
            ..Default::default()
        }
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn series(&mut self, series: &str) -> &mut Self {
        self.series = series.to_string();
        self
    }

    pub fn number(&mut self, number: f32) -> &mut Self {
        self.number = Some(number);
        self
    }

    pub fn volume(&mut self, volume: u32) -> &mut Self {
        self.volume = Some(volume);
        self
    }

    pub fn summary(&mut self, summary: &str) -> &mut Self {
        self.summary = Some(summary.to_string());
        self
    }

    pub fn date(&mut self, year: u16, month: u16, day: u8) -> &mut Self {
        self.year = Some(year);
        self.month = Some(month);
        self.day = Some(day);
        self
    }

    pub fn writer(&mut self, writer: &str) -> &mut Self {
        self.writer = Some(writer.to_string());
        self
    }

    pub fn penciller(&mut self, penciller: &str) -> &mut Self {
        self.penciller = Some(penciller.to_string());
        self
    }

    pub fn translator(&mut self, translator: &str) -> &mut Self {
        self.translator = Some(translator.to_string());
        self
    }

    pub fn publisher(&mut self, publisher: &str) -> &mut Self {
        self.publisher = Some(publisher.to_string());
        self
    }

    pub fn genre(&mut self, genre: &str) -> &mut Self {
        self.genre = Some(genre.to_string());
        self
    }

    pub fn tags(&mut self, tags: &str) -> &mut Self {
        self.tags = Some(tags.to_string());
        self
    }

    pub fn web(&mut self, web: &str) -> &mut Self {
        self.web = Some(web.to_string());
        self
    }

    pub fn page_count(&mut self, page_count: u32) -> &mut Self {
        self.page_count = Some(page_count);
        self
    }

    pub fn language_iso(&mut self, language_iso: &str) -> &mut Self {
        self.language_iso = Some(language_iso.to_string());
        self
    }

    pub fn manga(&mut self, manga: ComicInfoManga) -> &mut Self {
        self.manga = manga;
        self
    }

    pub fn age_rating(&mut self, age_rating: ComicInfoAgeRating) -> &mut Self {
        self.age_rating = age_rating;
        self
    }
}
