use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

pub struct PsiClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl PsiClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        PsiClient {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn get_report(&self, report_url: &str) -> Result<Value> {
        let params = [("url", report_url), ("key", &self.api_key)];
        let response = self
            .client
            .get(&self.base_url)
            .query(&params)
            .send()
            .await
            .context("Page Speed Insights request failed")?
            .json::<Value>()
            .await
            .context("Unable to parse JSON response")?;

        Ok(response)
    }
}
