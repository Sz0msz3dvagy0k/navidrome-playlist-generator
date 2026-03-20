use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    clients::lastfm::{LastfmClient, LastfmTrack},
    normalization::matching::match_score,
};

#[derive(sqlx::FromRow)]
struct DbSongMatch {
    id: Uuid,
    normalized_name: String,
    normalized_title: String,
}

fn best_song_match<'a>(track: &LastfmTrack, songs: &'a [DbSongMatch]) -> Option<&'a DbSongMatch> {
    songs
        .iter()
        .map(|candidate| {
            (
                candidate,
                match_score(
                    &track.artist,
                    &track.title,
                    &candidate.normalized_name,
                    &candidate.normalized_title,
                ),
            )
        })
        .filter(|(_, score)| *score >= 0.78)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(song, _)| song)
}

pub async fn ingest_lastfm_stats(client: &LastfmClient, pool: &PgPool) -> Result<()> {
    let recent_tracks = client.get_recent_tracks(500).await?;
    let top_tracks = client.get_top_tracks("1month", 500).await?;

    let songs: Vec<DbSongMatch> = sqlx::query_as(
        r#"
        SELECT s.id, a.normalized_name, s.normalized_title
        FROM songs s
        JOIN artists a ON a.id = s.artist_id
        "#,
    )
    .fetch_all(pool)
    .await?;

    for track in recent_tracks.iter().chain(top_tracks.iter()) {
        if let Some(song) = best_song_match(track, &songs) {
            if let Some(ts) = track.played_at_unix {
                if let Some(played_at) = DateTime::<Utc>::from_timestamp(ts, 0) {
                    sqlx::query(
                        r#"
                        INSERT INTO play_history (song_id, source, played_at)
                        VALUES ($1, 'lastfm', $2)
                        "#,
                    )
                    .bind(song.id)
                    .bind(played_at)
                    .execute(pool)
                    .await?;
                }
            }

            if track.play_count > 0 {
                sqlx::query(
                    r#"
                    INSERT INTO aggregated_stats (song_id, total_play_count, updated_at)
                    VALUES ($1, $2, NOW())
                    ON CONFLICT (song_id) DO UPDATE
                    SET total_play_count = GREATEST(aggregated_stats.total_play_count, EXCLUDED.total_play_count),
                        updated_at = NOW()
                    "#,
                )
                .bind(song.id)
                .bind(track.play_count)
                .execute(pool)
                .await?;
            }
        }
    }

    sqlx::query(
        r#"
        INSERT INTO aggregated_stats (
            song_id,
            total_play_count,
            recent_7d_count,
            recent_30d_count,
            last_played_at,
            updated_at
        )
        SELECT
            s.id,
            COALESCE(ast.total_play_count, 0) + COALESCE(hist.total_plays, 0),
            COALESCE(hist.recent_7d, 0),
            COALESCE(hist.recent_30d, 0),
            hist.last_played_at,
            NOW()
        FROM songs s
        LEFT JOIN aggregated_stats ast ON ast.song_id = s.id
        LEFT JOIN (
            SELECT
                song_id,
                COUNT(*)::BIGINT AS total_plays,
                COUNT(*) FILTER (WHERE played_at >= NOW() - INTERVAL '7 days')::BIGINT AS recent_7d,
                COUNT(*) FILTER (WHERE played_at >= NOW() - INTERVAL '30 days')::BIGINT AS recent_30d,
                MAX(played_at) AS last_played_at
            FROM play_history
            GROUP BY song_id
        ) hist ON hist.song_id = s.id
        ON CONFLICT (song_id) DO UPDATE
        SET total_play_count = EXCLUDED.total_play_count,
            recent_7d_count = EXCLUDED.recent_7d_count,
            recent_30d_count = EXCLUDED.recent_30d_count,
            last_played_at = EXCLUDED.last_played_at,
            updated_at = NOW()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
