use std::fmt::Display;

use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Serialize;
use serde_json::json;

use crate::komga::{KomgaBookResponse, KomgaSeriesResponse};

/// Manager for Komga API
#[derive(Clone)]
pub struct KomgaManager {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl Display for KomgaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base_url)
    }
}

impl KomgaManager {
    /// Create a new `KomgaManager`
    pub fn new(base_url: &str, api_key: &str) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().build()?;

        Ok(Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            client,
        })
    }

    /// Default headers
    fn headers(&self) -> anyhow::Result<HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key)?);
        Ok(headers)
    }

    /// Util method to build a GET request
    fn get(&self, path: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        Ok(self
            .client
            .get(format!("{}/{}", self.base_url, path))
            .headers(self.headers()?))
    }

    /// Util method to build a POST request
    fn post<T: Serialize>(&self, path: &str, body: &T) -> anyhow::Result<reqwest::RequestBuilder> {
        Ok(self
            .client
            .post(format!("{}/{}", self.base_url, path))
            .headers(self.headers()?)
            .json(body))
    }
}

impl KomgaManager {
    /// List all series available
    pub async fn list_series(&self) -> anyhow::Result<KomgaSeriesResponse> {
        let response = self
            .post("api/v1/series/list?unpaged=true", &json!({}))?
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Analyze a series
    ///
    /// This tells komga to perform a scan of the series refreshing metadata
    pub async fn analyze_series(&self, series_id: &str) -> anyhow::Result<()> {
        let response = self
            .post(&format!("api/v1/series/{series_id}/analyze"), &json!({}))?
            .send()
            .await?;

        let _ = response.text().await?;

        Ok(())
    }

    /// List all books in a series
    pub async fn list_books(&self, series_id: &str) -> anyhow::Result<KomgaBookResponse> {
        let json =
            json!({"condition":{"allOf":[{"seriesId":{"operator":"is","value": series_id}}]}});
        let response = self
            .post("api/v1/books/list?unpaged=true", &json)?
            .send()
            .await?;

        Ok(response.json().await?)
    }
}
