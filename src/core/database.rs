use std::str::FromStr;

use log::debug;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

use crate::{
    config::DATABASE_FILE_NAME, core::migrations::get_applied_migrations_count, error::AppError,
};

pub fn get_database_path() -> String {
    let data_dir = dirs::data_dir()
        .expect("failed to get data dir")
        .join("nameless-music-player");

    std::fs::create_dir_all(&data_dir).expect("failed to create data dir");

    format!("sqlite:{}", data_dir.join(DATABASE_FILE_NAME).display())
}

pub async fn create_pool() -> Result<SqlitePool, AppError> {
    let database_path = get_database_path();

    debug!("Connecting to database in location: {}", database_path);

    let options = SqliteConnectOptions::from_str(&database_path)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    debug!("Successfully connected to database.");

    Ok(pool)
}

/// Verifies the current schema version to prevent accidental downgrades.
///
/// # Errors
/// Returns an error if the current schema version is invalid (the expected version is less than current).
pub async fn check_schema_version(pool: &SqlitePool) -> Result<(), AppError> {
    let expected_schema_version = get_expected_schema_version();
    let current_schema_version = get_applied_migrations_count(&pool).await?;

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
