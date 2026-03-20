use anyhow::Result;
use sqlx::PgPool;

use crate::clients::lastfm::LastfmClient;

pub async fn ingest_lastfm_stats(_client: &LastfmClient, _pool: &PgPool) -> Result<()> {
    Ok(())
}
