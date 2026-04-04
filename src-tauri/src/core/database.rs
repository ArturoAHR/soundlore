use std::str::FromStr;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri::Manager;

use crate::{config::DATABASE_FILE_NAME, error::AppError};

pub fn get_database_path(app: &tauri::AppHandle) -> String {
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");

    format!("sqlite:{}", data_dir.join(DATABASE_FILE_NAME).display())
}

pub async fn create_pool(app: &tauri::AppHandle) -> Result<SqlitePool, AppError> {
    let database_path = get_database_path(app);
    let options = SqliteConnectOptions::from_str(&database_path)?.create_if_missing(true);
    Ok(SqlitePool::connect_with(options).await?)
}

/// Verifies the current schema version and determines if migrations are needed
///
/// # Errors
/// Returns an error if the current schema version is invalid (the expected version is less than current).
pub async fn check_schema_version(pool: &SqlitePool) -> Result<bool, AppError> {
    let expected_schema_version = get_expected_schema_version();
    let current_schema_version = get_current_schema_version(&pool).await?;

    if expected_schema_version < current_schema_version {
        return Err(AppError::DbDowngradeDetected {
            current: current_schema_version,
            expected: expected_schema_version,
        });
    }

    Ok(current_schema_version < expected_schema_version)
}

pub fn get_expected_schema_version() -> i64 {
    let migrator = sqlx::migrate!();

    migrator.migrations.len() as i64
}

pub async fn get_current_schema_version(pool: &SqlitePool) -> Result<i64, AppError> {
    let version = sqlx::query_scalar::<_, i64>("PRAGMA user_version")
        .fetch_one(pool)
        .await?;

    Ok(version)
}
