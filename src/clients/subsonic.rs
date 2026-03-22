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

    pub async fn find_playlist_id_by_name(&self, name: &str) -> Result<Option<String>> {
        let payload: serde_json::Value = self.get_json("getPlaylists", &[]).await?;
        let maybe_id = payload
            .get("subsonic-response")
            .and_then(|resp| resp.get("playlists"))
            .and_then(|playlists| playlists.get("playlist"))
            .and_then(|playlist_array| playlist_array.as_array())
            .and_then(|arr| {
                arr.iter().find_map(|p| {
                    let playlist_name = p.get("name")?.as_str()?;
                    if playlist_name == name {
                        p.get("id")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            });
        Ok(maybe_id)
    }

    pub async fn find_playlist_ids_for_cleanup(
        &self,
        target_name: &str,
        legacy_kind: &str,
    ) -> Result<Vec<String>> {
        let payload: serde_json::Value = self.get_json("getPlaylists", &[]).await?;
        let playlist_node = payload
            .get("subsonic-response")
            .and_then(|resp| resp.get("playlists"))
            .and_then(|playlists| playlists.get("playlist"));

        let legacy_suffix = format!(" - {}", legacy_kind);
        let mut ids = Vec::new();

        match playlist_node {
            Some(serde_json::Value::Array(arr)) => {
                for item in arr {
                    if let (Some(name), Some(id)) = (
                        item.get("name").and_then(|v| v.as_str()),
                        item.get("id").and_then(|v| v.as_str()),
                    ) {
                        let is_target = name == target_name;
                        let is_legacy =
                            name.starts_with("Daily Mix - ") && name.ends_with(&legacy_suffix);
                        if is_target || is_legacy {
                            ids.push(id.to_string());
                        }
                    }
                }
            }
            Some(serde_json::Value::Object(obj)) => {
                let name = obj.get("name").and_then(|v| v.as_str());
                let id = obj.get("id").and_then(|v| v.as_str());
                if let (Some(name), Some(id)) = (name, id) {
                    let is_target = name == target_name;
                    let is_legacy = name.starts_with("Daily Mix - ") && name.ends_with(&legacy_suffix);
                    if is_target || is_legacy {
                        ids.push(id.to_string());
                    }
                }
            }
            _ => {}
        }

        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    pub async fn delete_playlist(&self, playlist_id: &str) -> Result<()> {
        let _: serde_json::Value = self.get_json("deletePlaylist", &[("id", playlist_id)]).await?;
        Ok(())
    }

    pub async fn create_playlist(&self, name: &str, song_ids: &[String]) -> Result<String> {
        let mut url = format!(
            "{}/rest/createPlaylist.view?u={}&p={}&v=1.16.1&c=daily-playlist-generator&f=json&name={}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(&self.username),
            urlencoding::encode(&self.password),
            urlencoding::encode(name)
        );

        for song_id in song_ids {
            url.push_str("&songId=");
            url.push_str(&urlencoding::encode(song_id));
        }

        tracing::debug!("creating playlist with request: {}", url.replace(&self.password, "***"));

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("subsonic request failed")?
            .error_for_status()
            .context("subsonic error response")?;

        let payload: serde_json::Value = response
            .json()
            .await
            .context("invalid subsonic json payload")?;

        let id = payload
            .get("subsonic-response")
            .and_then(|resp| resp.get("playlist"))
            .and_then(|playlist| playlist.get("id"))
            .and_then(|id| id.as_str())
            .map(|s| s.to_string())
            .context("missing playlist id in createPlaylist response")?;
        Ok(id)
    }
}
