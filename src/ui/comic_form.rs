use tui_input::Input;

use crate::comic_info::{ComicInfo, ComicInfoAgeRating, ComicInfoManga};

/// Current comic selected on chapter list
pub struct ComicInfoForm {
    pub fields: Vec<(&'static str, Input)>, // label + input
    pub active_index: usize,
}

impl ComicInfoForm {
    pub fn new(info: &ComicInfo) -> Self {
        let fields = vec![
            ("Title", Input::new(info.title.clone())),
            ("Series*", Input::new(info.series.clone())),
            (
                "Number",
                Input::new(info.number.map(|n| n.to_string()).unwrap_or_default()),
            ),
            (
                "Volume",
                Input::new(info.volume.map(|v| v.to_string()).unwrap_or_default()),
            ),
            (
                "Summary*",
                Input::new(info.summary.clone().unwrap_or_default()),
            ),
            (
                "Year",
                Input::new(info.year.map(|y| y.to_string()).unwrap_or_default()),
            ),
            (
                "Month",
                Input::new(info.month.map(|m| m.to_string()).unwrap_or_default()),
            ),
            (
                "Day",
                Input::new(info.day.map(|d| d.to_string()).unwrap_or_default()),
            ),
            (
                "Writer*",
                Input::new(info.writer.clone().unwrap_or_default()),
            ),
            (
                "Penciller*",
                Input::new(info.penciller.clone().unwrap_or_default()),
            ),
            (
                "Translator",
                Input::new(info.translator.clone().unwrap_or_default()),
            ),
            (
                "Publisher*",
                Input::new(info.publisher.clone().unwrap_or_default()),
            ),
            ("Genre*", Input::new(info.genre.clone().unwrap_or_default())),
            ("Tags*", Input::new(info.tags.clone().unwrap_or_default())),
            ("Web*", Input::new(info.web.clone().unwrap_or_default())),
            (
                "Page Count",
                Input::new(info.page_count.map(|p| p.to_string()).unwrap_or_default()),
            ),
            (
                "Language ISO*",
                Input::new(info.language_iso.clone().unwrap_or_default()),
            ),
            ("Manga*", Input::new(info.manga.to_string())),
            ("Age Rating*", Input::new(info.age_rating.to_string())),
            (
                "Count*",
                Input::new(info.count.map(|c| c.to_string()).unwrap_or_default()),
            ),
        ];

        Self {
            fields,
            active_index: 0,
        }
    }

    pub fn next(&mut self) {
        self.active_index = (self.active_index + 1) % self.fields.len();
    }

    pub fn next_side(&mut self) {
        self.active_index = (self.active_index + 10) % self.fields.len();
    }

    pub fn prev(&mut self) {
        if self.active_index == 0 {
            self.active_index = self.fields.len() - 1;
        } else {
            self.active_index -= 1;
        }
    }

    pub fn prev_side(&mut self) {
        let step = 10 % self.fields.len();
        if self.active_index < step {
            self.active_index = self.fields.len() + self.active_index - step;
        } else {
            self.active_index -= step;
        }
    }

    pub fn active_input_mut(&mut self) -> &mut Input {
        &mut self.fields[self.active_index].1
    }

    pub fn to_comic_info(&self) -> ComicInfo {
        ComicInfo {
            title: self.fields[0].1.value().to_string(),
            series: self.fields[1].1.value().to_string(),
            number: parse_opt_f32(self.fields[2].1.value()),
            volume: parse_opt_u32(self.fields[3].1.value()),
            summary: parse_opt_string(self.fields[4].1.value()),
            year: parse_opt_u16(self.fields[5].1.value()),
            month: parse_opt_u16(self.fields[6].1.value()),
            day: parse_opt_u8(self.fields[7].1.value()),
            writer: parse_opt_string(self.fields[8].1.value()),
            penciller: parse_opt_string(self.fields[9].1.value()),
            translator: parse_opt_string(self.fields[10].1.value()),
            publisher: parse_opt_string(self.fields[11].1.value()),
            genre: parse_opt_string(self.fields[12].1.value()),
            tags: parse_opt_string(self.fields[13].1.value()),
            web: parse_opt_string(self.fields[14].1.value()),
            page_count: parse_opt_u32(self.fields[15].1.value()),
            language_iso: parse_opt_string(self.fields[16].1.value()),
            manga: parse_enum::<ComicInfoManga>(self.fields[17].1.value()).unwrap_or_default(),
            age_rating: parse_enum::<ComicInfoAgeRating>(self.fields[18].1.value())
                .unwrap_or_default(),
            count: parse_opt_u32(self.fields[19].1.value()),
        }
    }
}

fn parse_opt_string(s: &str) -> Option<String> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn parse_opt_f32(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok()
}

fn parse_opt_u32(s: &str) -> Option<u32> {
    s.trim().parse::<u32>().ok()
}

fn parse_opt_u16(s: &str) -> Option<u16> {
    s.trim().parse::<u16>().ok()
}

fn parse_opt_u8(s: &str) -> Option<u8> {
    s.trim().parse::<u8>().ok()
}

// For enum fields like Manga and AgeRating
fn parse_enum<T: std::str::FromStr>(s: &str) -> Option<T> {
    s.trim().parse::<T>().ok()
}

pub enum ComicFormState {
    Loading,
    Ready(ComicInfoForm),
}

impl ComicFormState {
    pub fn next(&mut self) {
        if let Self::Ready(comic) = self {
            comic.next();
        }
    }

    pub fn prev(&mut self) {
        if let Self::Ready(comic) = self {
            comic.prev();
        }
    }

    pub fn next_side(&mut self) {
        if let Self::Ready(comic) = self {
            comic.next_side();
        }
    }

    pub fn prev_side(&mut self) {
        if let Self::Ready(comic) = self {
            comic.prev_side();
        }
    }

    pub fn to_comic_info(&self) -> Option<ComicInfo> {
        match self {
            Self::Ready(comic) => Some(comic.to_comic_info()),
            Self::Loading => None,
        }
    }

    pub fn active_input_mut(&mut self) -> Option<&mut Input> {
        match self {
            Self::Ready(comic) => Some(comic.active_input_mut()),
            Self::Loading => None,
        }
    }
}
