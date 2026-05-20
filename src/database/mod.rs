use std::str::FromStr;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tracing::debug;

use crate::{
    config::DATABASE_FILE_NAME,
    database::migrations::{check_schema_version, run_pending_migrations},
    error::AppError,
};

pub mod migrations;

pub async fn initialize_database() -> Result<SqlitePool, AppError> {
    let pool = create_pool().await?;

    check_schema_version(&pool).await?;
    run_pending_migrations(&pool).await?;

    Ok(pool)
}

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
