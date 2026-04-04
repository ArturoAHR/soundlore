use std::path::PathBuf;

use serde::Serialize;
use symphonia::core::errors::Error as SymphoniaError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("app was downgraded but database is already at version {current} (app expects {expected}), please update the app.")]
    DbDowngradeDetected { current: i64, expected: i64 },

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
                AppError::Db(_) => "DB_ERROR",
                AppError::DbDowngradeDetected { .. } => "DB_VERSION_MISMATCH_ERROR",
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
