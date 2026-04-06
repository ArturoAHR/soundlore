use std::path::PathBuf;

use sqlx::SqlitePool;
use tauri_plugin_log::log::{info, warn};

use crate::error::AppError;
use crate::file::utils::find_track_files;
use crate::track::repository::upsert_track;
use crate::track::service::read_track_metadata;

pub async fn scan_files_in_directory(
    pool: &SqlitePool,
    directories: Vec<String>,
) -> Result<(), AppError> {
    info!("Scanning files in directories: {:?}", directories);

    let valid_directories = directories.into_iter().filter_map(|e| {
        let path = PathBuf::from(&e);

        if path.is_dir() {
            Some(path)
        } else {
            warn!("Invalid directory {:?}, skipping", &e);
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

    println!("{:#?}", processed_tracks);

    for track in processed_tracks {
        match upsert_track(pool, &track).await {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "Could not insert or update track {:?}: {}",
                    &track.file_path,
                    &e.to_string()
                )
            }
        }
    }

    Ok(())
}
