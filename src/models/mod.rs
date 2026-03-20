use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlaylistKind {
    Favorites,
    Rediscovery,
    Genre,
    Artist,
    SmartShuffle,
}

impl PlaylistKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Favorites => "Favorites",
            Self::Rediscovery => "Rediscovery",
            Self::Genre => "Genre",
            Self::Artist => "Artist",
            Self::SmartShuffle => "SmartShuffle",
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Song {
    pub id: Uuid,
    pub subsonic_id: String,
    pub artist_id: Uuid,
    pub title: String,
    pub normalized_title: String,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlayHistory {
    pub id: Uuid,
    pub song_id: Uuid,
    pub source: String,
    pub played_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AggregatedStat {
    pub song_id: Uuid,
    pub total_play_count: i64,
    pub recent_7d_count: i64,
    pub recent_30d_count: i64,
    pub last_played_at: Option<DateTime<Utc>>,
    pub score_cache: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GeneratedPlaylist {
    pub id: Uuid,
    pub date: NaiveDate,
    pub kind: String,
    pub name: String,
    pub song_ids: Vec<Uuid>,
    pub navidrome_playlist_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
