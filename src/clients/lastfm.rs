use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct LastfmTrack {
    pub artist: String,
    pub title: String,
    pub play_count: i64,
    pub played_at_unix: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LastfmTrackListResponse {
    recenttracks: Option<LastfmRecentTracks>,
    toptracks: Option<LastfmTopTracks>,
}

#[derive(Debug, Deserialize)]
struct LastfmRecentTracks {
    #[serde(rename = "track", default)]
    tracks: Vec<LastfmRawTrack>,
}

#[derive(Debug, Deserialize)]
struct LastfmTopTracks {
    #[serde(rename = "track", default)]
    tracks: Vec<LastfmRawTrack>,
}

#[derive(Debug, Deserialize)]
struct LastfmRawTrack {
    name: String,
    #[serde(default)]
    playcount: Option<String>,
    artist: LastfmArtist,
    #[serde(rename = "date", default)]
    date: Option<LastfmDate>,
}

#[derive(Debug, Deserialize)]
struct LastfmArtist {
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct LastfmDate {
    uts: String,
}

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

    async fn get_method<T>(&self, method: &str, extra: &[(&str, &str)]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut params: Vec<(&str, &str)> = vec![
            ("method", method),
            ("user", &self.username),
            ("api_key", &self.api_key),
            ("format", "json"),
        ];
        params.extend_from_slice(extra);

        let response = self
            .http
            .get("https://ws.audioscrobbler.com/2.0/")
            .query(&params)
            .send()
            .await
            .context("lastfm request failed")?
            .error_for_status()
            .context("lastfm error response")?;

        let payload = response
            .json::<T>()
            .await
            .context("invalid lastfm json payload")?;

        Ok(payload)
    }

    pub async fn ping(&self) -> Result<()> {
        let _: serde_json::Value = self
            .get_method("user.getrecenttracks", &[("limit", "1")])
            .await?;
        Ok(())
    }

    pub async fn get_recent_tracks(&self, limit: usize) -> Result<Vec<LastfmTrack>> {
        let limit_str = limit.to_string();
        let response: LastfmTrackListResponse = self
            .get_method("user.getrecenttracks", &[("limit", &limit_str)])
            .await?;

        let tracks = response
            .recenttracks
            .map(|v| {
                v.tracks
                    .into_iter()
                    .map(|raw| LastfmTrack {
                        artist: raw.artist.text,
                        title: raw.name,
                        play_count: 1,
                        played_at_unix: raw.date.and_then(|d| d.uts.parse::<i64>().ok()),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(tracks)
    }

    pub async fn get_top_tracks(&self, period: &str, limit: usize) -> Result<Vec<LastfmTrack>> {
        let limit_str = limit.to_string();
        let response: LastfmTrackListResponse = self
            .get_method(
                "user.gettoptracks",
                &[("period", period), ("limit", &limit_str)],
            )
            .await?;

        let tracks = response
            .toptracks
            .map(|v| {
                v.tracks
                    .into_iter()
                    .map(|raw| LastfmTrack {
                        artist: raw.artist.text,
                        title: raw.name,
                        play_count: raw
                            .playcount
                            .as_deref()
                            .and_then(|c| c.parse::<i64>().ok())
                            .unwrap_or(0),
                        played_at_unix: raw.date.and_then(|d| d.uts.parse::<i64>().ok()),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(tracks)
    }
}
