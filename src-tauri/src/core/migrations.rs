use std::error::Error;

use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), Box<dyn Error>> {
    sqlx::migrate!().run(pool).await?;

    Ok(())
}
