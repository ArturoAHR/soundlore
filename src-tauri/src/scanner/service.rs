use std::path::PathBuf;

use tauri_plugin_log::log::{info, warn};

use crate::error::AppError;
use crate::file::utils::find_track_files;
use crate::tracks::service::upsert_track;

pub fn scan_files_in_directory(directories: Vec<String>) -> Result<(), AppError> {
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
        match upsert_track(&track_file_path) {
            Ok(track) => processed_tracks.push(track),
            Err(e) => {
                warn!(
                    "Could not add or update track {:?}: {}",
                    &track_file_path,
                    &e.to_string()
                )
            }
        };
    }

    Ok(())
}
