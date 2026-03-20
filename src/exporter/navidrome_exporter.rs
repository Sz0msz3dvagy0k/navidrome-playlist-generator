use anyhow::Result;

use crate::{clients::subsonic::SubsonicClient, generator::playlist_generator::PlaylistDraft};

pub async fn export_playlist(_client: &SubsonicClient, _playlist: &PlaylistDraft) -> Result<String> {
    Ok(String::new())
}
