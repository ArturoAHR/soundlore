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
        match read_track_metadata(&track_file_path) {
            Ok(track) => processed_tracks.push(track),
            Err(e) => {
                warn!(
                    "Could not read track metadata for file {:?}: {}",
                    &track_file_path,
                    &e.to_string()
                )
            }
        };
    }

    upsert_tracks_batch(pool, processed_tracks.as_slice()).await
}
