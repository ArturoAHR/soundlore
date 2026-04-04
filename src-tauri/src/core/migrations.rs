use std::{error::Error, str::FromStr};

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri_plugin_log::log::info;

use crate::core::database::get_database_path;

pub async fn run_migrations(app: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let database_path = get_database_path(app);

    info!("Running migrations on database at {:?}", database_path);

    let options = SqliteConnectOptions::from_str(&database_path)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    Ok(sqlx::migrate!().run(&pool).await?)
}
