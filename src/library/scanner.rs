use std::path::PathBuf;

use sqlx::SqlitePool;
use tracing::{error, info, instrument, warn};

use crate::error::AppError;
use crate::file::utils::find_track_files;
use crate::track::metadata::read_track_metadata;
use crate::track::repository::upsert_tracks_batch;

#[instrument(skip_all, fields(directory_count = directories.len()))]
pub async fn scan_files_in_directory(
    pool: SqlitePool,
    directories: Vec<PathBuf>,
) -> Result<(), AppError> {
    info!(?directories, "Scanning files in directories");

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

    info!(
        file_count = track_file_paths.len(),
        "Discovered candidate files"
    );

    let candidate_track_count = track_file_paths.len();
    let mut processed_tracks = vec![];

    for track_file_path in track_file_paths {
        let track_metadata_thread_result = tokio::task::spawn_blocking(move || {
            (read_track_metadata(&track_file_path), track_file_path)
        })
        .await;

        match track_metadata_thread_result {
            Ok((Ok(track_metadata), _)) => processed_tracks.push(track_metadata),
            Ok((Err(error), track_file_path)) => {
                error!(
                    "Could not read track metadata for file {:?}: {}",
                    &track_file_path,
                    &error.to_string()
                )
            }
            Err(error) => {
                error!("Metadata read panicked: {}", &error.to_string())
            }
        };
    }

    info!(
        successfully_processed = processed_tracks.len(),
        failed_to_process = (candidate_track_count - processed_tracks.len()),
        "Track metadata extraction complete"
    );

    upsert_tracks_batch(pool, processed_tracks.as_slice()).await
}
