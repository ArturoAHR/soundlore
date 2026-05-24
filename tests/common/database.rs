use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use crate::common::async_runtime::block_on;

pub async fn get_database_pool() -> SqlitePool {
    let connection_options = SqliteConnectOptions::new().in_memory(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(connection_options)
        .await
        .expect("Could not create in-memory database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Could not run migrations");

    pool
}

/// Meant to be used with emulator tests as those cannot use the `tokio::test` macro.
pub fn get_database_pool_sync() -> SqlitePool {
    let connection_options = SqliteConnectOptions::new().in_memory(true);

    let pool = block_on(
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connection_options),
    )
    .expect("Could not create in-memory database");

    block_on(sqlx::migrate!().run(&pool)).expect("Could not run migrations");

    pool
}
