use std::path::PathBuf;

use sea_query::error::Error as SeaQueryError;
use sqlx::migrate::MigrateError;
use symphonia::core::errors::Error as SymphoniaError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("database migration error: {0}")]
    DatabaseMigration(#[from] MigrateError),

    #[error("app was downgraded but database is already at version {current} (app expects {expected}), please update the app.")]
    DatabaseDowngradeDetected { current: i64, expected: i64 },

    #[error("database query generation error: {0}")]
    DatabaseQueryGeneration(#[from] SeaQueryError),

    #[error("track not found: {path}")]
    TrackNotFound { path: PathBuf },

    #[error("playlist not found: {name}")]
    PlaylistNotFound { name: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("playback error: {0}")]
    Playback(#[from] SymphoniaError),
}
