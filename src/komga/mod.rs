use serde::{Deserialize, Serialize};

pub mod book;
pub mod series;

#[derive(Debug, Deserialize)]
struct RawAuthor {
    name: String,
    role: String,
}

// Done so only allowed types can be used
pub trait KomgaItem {}
impl KomgaItem for series::KomgaSeries {}
impl KomgaItem for book::KomgaBook {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KomgaResponse<T: KomgaItem> {
    pub total_elements: i64,
    pub total_pages: i32,
    pub content: Vec<T>,
}

pub type KomgaSeriesResponse = KomgaResponse<series::KomgaSeries>;
pub type KomgaBookResponse = KomgaResponse<book::KomgaBook>;
