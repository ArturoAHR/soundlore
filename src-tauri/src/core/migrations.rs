use sqlx::SqlitePool;
use tauri_plugin_log::log::info;

use crate::error::AppError;

/// Performs a sanity check on already applied migrations and runs any pending migrations.
///
/// # Errors
/// Returns an error if any of the previously applied migrations is altered.
pub async fn run_pending_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    info!("Running database migrations.");

    sqlx::migrate!().run(pool).await?;

    Ok(())
}

pub async fn get_applied_migrations_count(pool: &SqlitePool) -> Result<i64, AppError> {
    let table_exists: bool = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?
        > 0;

    if !table_exists {
        return Ok(0);
    }

    let applied_migrations_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;

    Ok(applied_migrations_count)
}
