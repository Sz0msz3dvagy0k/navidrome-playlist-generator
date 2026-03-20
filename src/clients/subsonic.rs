use anyhow::Result;
use reqwest::Client;

#[derive(Clone)]
pub struct SubsonicClient {
    pub http: Client,
    pub base_url: String,
    pub username: String,
    pub password: String,
}

impl SubsonicClient {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self {
            http: Client::new(),
            base_url,
            username,
            password,
        }
    }

    pub async fn ping(&self) -> Result<()> {
        let _ = &self.http;
        Ok(())
    }
}
