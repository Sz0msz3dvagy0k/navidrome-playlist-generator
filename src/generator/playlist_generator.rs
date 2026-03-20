use chrono::NaiveDate;
use uuid::Uuid;

use crate::models::PlaylistKind;

#[derive(Debug, Clone)]
pub struct PlaylistDraft {
    pub date: NaiveDate,
    pub kind: PlaylistKind,
    pub name: String,
    pub song_ids: Vec<Uuid>,
}

pub fn playlist_name(date: NaiveDate, kind: PlaylistKind) -> String {
    format!("Daily Mix - {} - {}", date, kind.as_str())
}
