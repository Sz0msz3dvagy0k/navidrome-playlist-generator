use std::net::SocketAddr;

use anyhow::Result;
use axum::serve;
use daily_playlist_generator::{
    api,
    clients::{lastfm::LastfmClient, subsonic::SubsonicClient},
    config::AppConfig,
    db,
    scheduler::daily_job,
};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = AppConfig::from_env()?;
    let pool = db::create_pool(&cfg.database_url).await?;
    db::migrate(&pool).await?;

    let subsonic = SubsonicClient::new(
        cfg.subsonic_base_url.clone(),
        cfg.subsonic_username.clone(),
        cfg.subsonic_password.clone(),
    );
    let lastfm = LastfmClient::new(cfg.lastfm_api_key.clone(), cfg.lastfm_username.clone());

    // Run one immediate cycle, then continue with daily cadence.
    let scheduler_pool = pool.clone();
    let scheduler_subsonic = subsonic.clone();
    let scheduler_lastfm = lastfm.clone();
    let playlist_size = cfg.playlist_size;
    tokio::spawn(async move {
        daily_job::scheduler_loop(
            scheduler_pool,
            scheduler_subsonic,
            scheduler_lastfm,
            playlist_size,
        )
        .await;
    });

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("service listening on {}", addr);
    serve(listener, api::router()).await?;

    Ok(())
}
