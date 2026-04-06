use std::path::PathBuf;

use sea_query::error::Error as SeaQueryError;
use serde::Serialize;
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

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ErrorPayload {
            code: &'static str,
            message: String,
        }

        ErrorPayload {
            code: match self {
                AppError::Database(_) => "DATABASE_ERROR",
                AppError::DatabaseMigration(_) => "DATABASE_MIGRATION_ERROR",
                AppError::DatabaseDowngradeDetected { .. } => "DATABASE_VERSION_MISMATCH_ERROR",
                AppError::DatabaseQueryGeneration(_) => "DATABASE_QUERY_GENERATION_ERROR",
                AppError::TrackNotFound { .. } => "TRACK_NOT_FOUND",
                AppError::PlaylistNotFound { .. } => "PLAYLIST_NOT_FOUND",
                AppError::Io(_) => "IO_ERROR",
                AppError::Playback(_) => "PLAYBACK_ERROR",
            },
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}
