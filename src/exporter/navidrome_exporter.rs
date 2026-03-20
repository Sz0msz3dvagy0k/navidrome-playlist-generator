use anyhow::Result;
use sqlx::PgPool;

use crate::{clients::subsonic::SubsonicClient, generator::playlist_generator::PlaylistDraft};

pub async fn export_playlist(
    client: &SubsonicClient,
    pool: &PgPool,
    playlist: &PlaylistDraft,
) -> Result<String> {
    let subsonic_song_ids: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT subsonic_id
        FROM songs
        WHERE id = ANY($1)
        "#,
    )
    .bind(&playlist.song_ids)
    .fetch_all(pool)
    .await?;

    if let Some(existing_id) = client.find_playlist_id_by_name(&playlist.name).await? {
        client.delete_playlist(&existing_id).await?;
    }

    let navidrome_id = client
        .create_playlist(&playlist.name, &subsonic_song_ids)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO generated_playlists (date, kind, name, song_ids, navidrome_playlist_id)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (date, kind) DO UPDATE
        SET name = EXCLUDED.name,
            song_ids = EXCLUDED.song_ids,
            navidrome_playlist_id = EXCLUDED.navidrome_playlist_id,
            updated_at = NOW()
        "#,
    )
    .bind(playlist.date)
    .bind(playlist.kind.as_str())
    .bind(&playlist.name)
    .bind(&playlist.song_ids)
    .bind(&navidrome_id)
    .execute(pool)
    .await?;

    Ok(navidrome_id)
}
