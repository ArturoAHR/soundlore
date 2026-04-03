use std::{error::Error, str::FromStr};

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub async fn run_migrations() -> Result<(), Box<dyn Error>> {
    let options = SqliteConnectOptions::from_str("sqlite:data.db")?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    Ok(sqlx::migrate!().run(&pool).await?)
}
