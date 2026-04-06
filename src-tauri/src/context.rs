use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{
    error::AppError,
    scanner::service::{ScannerService, ScannerServiceImpl},
    track::{
        repository::{TrackRepository, TrackRepositoryImpl},
        service::{TrackService, TrackServiceImpl},
    },
};

pub struct Context {
    pub track_service: Arc<dyn TrackService>,
    pub scanner_service: Arc<dyn ScannerService>,
}

impl Context {
    pub async fn init(pool: SqlitePool) -> Result<Self, AppError> {
        let pool = Arc::new(pool);
        let track_repository: Arc<dyn TrackRepository> =
            Arc::new(TrackRepositoryImpl::new(Arc::clone(&pool)));
        let track_service: Arc<dyn TrackService> = Arc::new(TrackServiceImpl::new());
        let scanner_service = Arc::new(ScannerServiceImpl::new(
            Arc::clone(&track_service),
            Arc::clone(&track_repository),
        ));

        Ok(Self {
            track_service,
            scanner_service,
        })
    }
}
