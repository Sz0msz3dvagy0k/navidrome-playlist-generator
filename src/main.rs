use std::net::SocketAddr;

use anyhow::Result;
use axum::serve;
use daily_playlist_generator::{api, config::AppConfig, db};
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

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("service listening on {}", addr);
    serve(listener, api::router()).await?;

    Ok(())
}
