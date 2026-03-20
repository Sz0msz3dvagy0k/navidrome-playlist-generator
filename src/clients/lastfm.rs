use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct LastfmClient {
    pub http: Client,
    pub api_key: String,
    pub username: String,
}

impl LastfmClient {
    pub fn new(api_key: String, username: String) -> Self {
        Self {
            http: Client::new(),
            api_key,
            username,
        }
    }

    pub async fn ping(&self) -> Result<()> {
        let _ = &self.http;
        Ok(())
    }
}
