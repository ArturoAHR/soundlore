use std::str::FromStr;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing::{debug, error, info, instrument};

use crate::{
    config::DATABASE_FILE_NAME,
    database::migrations::{check_schema_version, run_pending_migrations},
    error::AppError,
    file::utils::get_application_directory_name,
};

pub mod migrations;

#[instrument]
pub async fn initialize_database() -> Result<SqlitePool, AppError> {
    info!("Initializing Database");

    let pool = create_pool().await?;

    check_schema_version(&pool).await?;
    run_pending_migrations(&pool).await?;

    info!("Database initialized");

    Ok(pool)
}

pub fn get_database_path() -> String {
    let application_directory_name = get_application_directory_name();

    let data_directory = dirs::data_dir()
        .unwrap_or_else(|| {
            error!("Failed to get user data directory.");

            panic!("Failed to get data directory");
        })
        .join(application_directory_name);

    std::fs::create_dir_all(&data_directory).expect("Failed to create data directory");

    format!(
        "sqlite:{}",
        data_directory.join(DATABASE_FILE_NAME).display()
    )
}

#[instrument]
pub async fn create_pool() -> Result<SqlitePool, AppError> {
    let database_path = get_database_path();

    debug!("Connecting to database in location: {}", database_path);

    let options = SqliteConnectOptions::from_str(&database_path)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    debug!("Successfully connected to database.");

    Ok(pool)
}
