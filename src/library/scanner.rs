use std::path::PathBuf;

use sqlx::SqlitePool;
use tracing::{info, instrument, warn};

use crate::error::AppError;
use crate::file::utils::find_track_files;
use crate::track::metadata::read_track_metadata;
use crate::track::repository::upsert_tracks_batch;

#[instrument(skip_all)]
pub async fn scan_files_in_directory(
    pool: &SqlitePool,
    directories: Vec<PathBuf>,
) -> Result<(), AppError> {
    info!("Scanning files in directories: {:?}", directories);

    let valid_directories = directories.into_iter().filter_map(|path| {
        if path.is_dir() {
            Some(path)
        } else {
            warn!("Invalid directory {:?}, skipping", &path);
            None
        }
    });

    let mut track_file_paths = vec![];

    for directory in valid_directories {
        track_file_paths.extend(find_track_files(&directory))
    }

    let mut processed_tracks = vec![];

    for track_file_path in track_file_paths {
        let track_metadata =
            tokio::task::spawn_blocking(move || read_track_metadata(&track_file_path)).await??;

        processed_tracks.push(track_metadata);
    }

    upsert_tracks_batch(pool, processed_tracks.as_slice()).await
}
