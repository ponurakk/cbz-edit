use std::fmt::Display;

use serde::Serialize;
use serde_json::json;

/// Manager for Komga API
#[derive(Clone)]
pub struct KomfManager {
    base_url: String,
    client: reqwest::Client,
}

impl Display for KomfManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base_url)
    }
}

impl KomfManager {
    /// Create a new `KomgaManager`
    pub fn new(base_url: &str) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().build()?;

        Ok(Self {
            base_url: base_url.to_string(),
            client,
        })
    }

    /// Util method to build a POST request
    fn post<T: Serialize>(&self, path: &str, body: &T) -> reqwest::RequestBuilder {
        self.client
            .post(format!("{}/{}", self.base_url, path))
            .json(body)
    }
}

impl KomfManager {
    /// Identify series with komf
    pub async fn identify(&self, library_id: &str, series_id: &str) -> anyhow::Result<()> {
        self.post(
            &format!("komga/match/library/{library_id}/series/{series_id}"),
            &json!({}),
        )
        .send()
        .await?;

        Ok(())
    }
}
