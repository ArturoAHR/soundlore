use std::error::Error;

use sqlx::SqlitePool;
use tauri_plugin_log::log::info;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), Box<dyn Error>> {
    info!("Running database migrations.");

    sqlx::migrate!().run(pool).await?;

    Ok(())
}
