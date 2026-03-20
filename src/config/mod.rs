use std::env;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub subsonic_base_url: String,
    pub subsonic_username: String,
    pub subsonic_password: String,
    pub lastfm_api_key: String,
    pub lastfm_username: String,
    pub playlist_size: usize,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("invalid APP_PORT")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            subsonic_base_url: env::var("SUBSONIC_BASE_URL")
                .context("SUBSONIC_BASE_URL is required")?,
            subsonic_username: env::var("SUBSONIC_USERNAME")
                .context("SUBSONIC_USERNAME is required")?,
            subsonic_password: env::var("SUBSONIC_PASSWORD")
                .context("SUBSONIC_PASSWORD is required")?,
            lastfm_api_key: env::var("LASTFM_API_KEY").context("LASTFM_API_KEY is required")?,
            lastfm_username: env::var("LASTFM_USERNAME").context("LASTFM_USERNAME is required")?,
            playlist_size: env::var("PLAYLIST_SIZE")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .context("invalid PLAYLIST_SIZE")?,
        })
    }
}
