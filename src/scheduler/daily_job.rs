use anyhow::Result;
use chrono::{Days, Utc};
use sqlx::PgPool;
use tokio::time::{sleep, Duration};

use crate::{
    clients::{lastfm::LastfmClient, subsonic::SubsonicClient},
    exporter::navidrome_exporter::export_playlist,
    generator::playlist_generator::generate_daily_playlists,
    ingestion::{lastfm_ingestion::ingest_lastfm_stats, subsonic_ingestion::ingest_subsonic_metadata},
};

pub async fn run_daily_job(
    pool: &PgPool,
    subsonic: &SubsonicClient,
    lastfm: &LastfmClient,
    playlist_size: usize,
) -> Result<()> {
    ingest_subsonic_metadata(subsonic, pool).await?;
    ingest_lastfm_stats(lastfm, pool).await?;

    let today = Utc::now().date_naive();
    let playlists = generate_daily_playlists(pool, today, playlist_size).await?;
    for playlist in playlists {
        export_playlist(subsonic, pool, &playlist).await?;
    }

    Ok(())
}

pub async fn scheduler_loop(
    pool: PgPool,
    subsonic: SubsonicClient,
    lastfm: LastfmClient,
    playlist_size: usize,
) {
    loop {
        if let Err(error) = run_daily_job(&pool, &subsonic, &lastfm, playlist_size).await {
            tracing::error!("daily job failed: {error:#}");
        } else {
            tracing::info!("daily playlist job completed");
        }

        let now = Utc::now();
        let tomorrow = now
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap_or_else(|| now.date_naive());
        let next = tomorrow
            .and_hms_opt(0, 5, 0)
            .unwrap_or_else(|| now.naive_utc());
        let wait = (next - now.naive_utc())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(24 * 60 * 60));

        sleep(wait).await;
    }
}
