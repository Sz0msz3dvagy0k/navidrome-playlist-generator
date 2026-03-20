use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::subsonic::SubsonicClient;
use crate::utils::text::normalize_text;

pub async fn ingest_subsonic_metadata(client: &SubsonicClient, pool: &PgPool) -> Result<()> {
    let artists = client.get_artists().await?;

    for artist in artists {
        let normalized_artist = normalize_text(&artist.name);

        let artist_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO artists (name, normalized_name)
            VALUES ($1, $2)
            ON CONFLICT (normalized_name) DO UPDATE
            SET name = EXCLUDED.name,
                updated_at = NOW()
            RETURNING id
            "#,
        )
        .bind(&artist.name)
        .bind(&normalized_artist)
        .fetch_one(pool)
        .await?;

        let albums = client.get_artist_albums(&artist.id).await?;
        for album in albums {
            let songs = client.get_album_songs(&album.id).await?;
            for song in songs {
                let normalized_title = normalize_text(&song.title);

                sqlx::query(
                    r#"
                    INSERT INTO songs (
                        subsonic_id, artist_id, title, normalized_title, album, genre, year, duration_seconds
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (subsonic_id) DO UPDATE
                    SET artist_id = EXCLUDED.artist_id,
                        title = EXCLUDED.title,
                        normalized_title = EXCLUDED.normalized_title,
                        album = EXCLUDED.album,
                        genre = EXCLUDED.genre,
                        year = EXCLUDED.year,
                        duration_seconds = EXCLUDED.duration_seconds,
                        updated_at = NOW()
                    "#,
                )
                .bind(&song.id)
                .bind(artist_id)
                .bind(&song.title)
                .bind(&normalized_title)
                .bind(song.album.or_else(|| Some(album.name.clone())))
                .bind(song.genre)
                .bind(song.year)
                .bind(song.duration)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}
