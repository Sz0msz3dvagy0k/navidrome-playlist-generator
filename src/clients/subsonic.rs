use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SubsonicArtistSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubsonicAlbumSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubsonicSong {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub duration: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ArtistsEnvelope {
    #[serde(rename = "subsonic-response")]
    pub response: ArtistsResponse,
}

#[derive(Debug, Deserialize)]
struct ArtistsResponse {
    pub artists: Option<ArtistIndexes>,
}

#[derive(Debug, Deserialize)]
struct ArtistIndexes {
    #[serde(rename = "index", default)]
    pub indexes: Vec<ArtistIndex>,
}

#[derive(Debug, Deserialize)]
struct ArtistIndex {
    #[serde(rename = "artist", default)]
    pub artists: Vec<SubsonicArtistSummary>,
}

#[derive(Debug, Deserialize)]
struct ArtistDetailEnvelope {
    #[serde(rename = "subsonic-response")]
    pub response: ArtistDetailResponse,
}

#[derive(Debug, Deserialize)]
struct ArtistDetailResponse {
    pub artist: Option<ArtistDetail>,
}

#[derive(Debug, Deserialize)]
struct ArtistDetail {
    #[serde(rename = "album", default)]
    pub albums: Vec<SubsonicAlbumSummary>,
}

#[derive(Debug, Deserialize)]
struct AlbumDetailEnvelope {
    #[serde(rename = "subsonic-response")]
    pub response: AlbumDetailResponse,
}

#[derive(Debug, Deserialize)]
struct AlbumDetailResponse {
    pub album: Option<AlbumDetail>,
}

#[derive(Debug, Deserialize)]
struct AlbumDetail {
    #[serde(rename = "song", default)]
    pub songs: Vec<SubsonicSong>,
}

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

    async fn get_json<T>(&self, endpoint: &str, extra: &[(&str, &str)]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut params: Vec<(&str, &str)> = vec![
            ("u", &self.username),
            ("p", &self.password),
            ("v", "1.16.1"),
            ("c", "daily-playlist-generator"),
            ("f", "json"),
        ];
        params.extend_from_slice(extra);

        let url = format!("{}/rest/{}.view", self.base_url.trim_end_matches('/'), endpoint);
        let response = self
            .http
            .get(url)
            .query(&params)
            .send()
            .await
            .context("subsonic request failed")?
            .error_for_status()
            .context("subsonic error response")?;

        let payload = response
            .json::<T>()
            .await
            .context("invalid subsonic json payload")?;

        Ok(payload)
    }

    pub async fn ping(&self) -> Result<()> {
        let _: serde_json::Value = self.get_json("ping", &[]).await?;
        Ok(())
    }

    pub async fn get_artists(&self) -> Result<Vec<SubsonicArtistSummary>> {
        let payload: ArtistsEnvelope = self.get_json("getArtists", &[]).await?;
        let artists = payload
            .response
            .artists
            .map(|group| {
                group
                    .indexes
                    .into_iter()
                    .flat_map(|idx| idx.artists)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(artists)
    }

    pub async fn get_artist_albums(&self, artist_id: &str) -> Result<Vec<SubsonicAlbumSummary>> {
        let payload: ArtistDetailEnvelope = self
            .get_json("getArtist", &[("id", artist_id)])
            .await?;
        Ok(payload.response.artist.map(|a| a.albums).unwrap_or_default())
    }

    pub async fn get_album_songs(&self, album_id: &str) -> Result<Vec<SubsonicSong>> {
        let payload: AlbumDetailEnvelope = self.get_json("getAlbum", &[("id", album_id)]).await?;
        Ok(payload.response.album.map(|a| a.songs).unwrap_or_default())
    }
}
