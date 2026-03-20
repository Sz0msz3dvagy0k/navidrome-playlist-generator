use anyhow::Result;
use sqlx::PgPool;

use crate::clients::subsonic::SubsonicClient;

pub async fn ingest_subsonic_metadata(_client: &SubsonicClient, _pool: &PgPool) -> Result<()> {
    Ok(())
}
