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
    tracing::info!("starting daily job: ingestion → scoring → generation → export");

    tracing::info!("ingesting subsonic metadata");
    ingest_subsonic_metadata(subsonic, pool).await?;

    tracing::info!("ingesting lastfm stats");
    ingest_lastfm_stats(lastfm, pool).await?;

    let today = Utc::now().date_naive();
    tracing::info!("generating playlists for {}", today);
    let playlists = generate_daily_playlists(pool, today, playlist_size).await?;

    tracing::info!("exporting {} playlists to navidrome", playlists.len());
    for playlist in playlists {
        match export_playlist(subsonic, pool, &playlist).await {
            Ok(id) => tracing::info!("exported playlist {} -> {}", playlist.name, id),
            Err(e) => tracing::error!("failed to export playlist {}: {}", playlist.name, e),
        }
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
