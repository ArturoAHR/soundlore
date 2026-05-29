use std::{path::PathBuf, sync::Arc};

use sea_query::error::Error as SeaQueryError;
use sqlx::migrate::MigrateError;
use thiserror::Error;
use tokio::task::JoinError;

use crate::{
    playback::{
        engine::PlaybackEngineError, pipeline::AudioPipelineError, PlaybackControllerError,
    },
    track::metadata::TrackPropertiesReadError,
};

#[derive(Debug, Error, Clone)]
pub enum AppError {
    #[error("database error - {0}")]
    Database(Arc<sqlx::Error>),

    #[error("database migration error - {0}")]
    DatabaseMigration(Arc<MigrateError>),

    #[error("app was downgraded but database is already at version {current} (app expects {expected}), please update the app.")]
    DatabaseDowngradeDetected { current: i64, expected: i64 },

    #[error("database query generation error - {0}")]
    DatabaseQueryGeneration(Arc<SeaQueryError>),

    #[error("track not found: {path}")]
    TrackNotFound { path: PathBuf },

    #[error("playlist not found: {name}")]
    PlaylistNotFound { name: String },

    #[error("io error - {0}")]
    Io(Arc<std::io::Error>),

    #[error("track read error - {0}")]
    TrackPropertiesRead(#[from] TrackPropertiesReadError),

    #[error("thread join error - {0}")]
    ThreadJoinFailed(Arc<JoinError>),

    #[error("audio pipeline error - {0}")]
    AudioPipeline(#[from] AudioPipelineError),

    #[error("playback error - {0}")]
    PlaybackEngine(#[from] PlaybackEngineError),

    #[error("playback controller error - {0}")]
    PlaybackController(#[from] PlaybackControllerError),
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(Arc::new(error))
    }
}

impl From<MigrateError> for AppError {
    fn from(error: MigrateError) -> Self {
        Self::DatabaseMigration(Arc::new(error))
    }
}

impl From<SeaQueryError> for AppError {
    fn from(error: SeaQueryError) -> Self {
        Self::DatabaseQueryGeneration(Arc::new(error))
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(Arc::new(error))
    }
}

impl From<JoinError> for AppError {
    fn from(error: JoinError) -> Self {
        Self::ThreadJoinFailed(Arc::new(error))
    }
}
