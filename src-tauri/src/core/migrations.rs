use std::{error::Error, str::FromStr};

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

use crate::core::database::get_database_path;

pub async fn run_migrations(app: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let options = SqliteConnectOptions::from_str(&get_database_path(app))?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    Ok(sqlx::migrate!().run(&pool).await?)
}
