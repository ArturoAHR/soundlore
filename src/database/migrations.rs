use sqlx::SqlitePool;
use tracing::{debug, info};

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

/// Verifies the current schema version to prevent accidental downgrades.
///
/// # Errors
/// Returns an error if the current schema version is invalid (the expected version is less than current).
pub async fn check_schema_version(pool: &SqlitePool) -> Result<(), AppError> {
    let expected_schema_version = get_expected_schema_version();
    let current_schema_version = get_applied_migrations_count(pool).await?;

    if expected_schema_version < current_schema_version {
        return Err(AppError::DatabaseDowngradeDetected {
            current: current_schema_version,
            expected: expected_schema_version,
        });
    }

    Ok(())
}

pub fn get_expected_schema_version() -> i64 {
    let migrator = sqlx::migrate!();

    let version = migrator.migrations.len() as i64;

    debug!("Expected Schema Version: {}", version);

    version
}
