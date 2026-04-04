use std::path::{Path, PathBuf};

use tauri_plugin_log::log::{debug, trace};
use walkdir::WalkDir;

pub fn find_track_files(root: &Path) -> Vec<PathBuf> {
    debug!("Finding music files in {:?}", root);

    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_supported_track_file(e.path()))
        .map(|e| e.path().to_owned())
        .collect()
}

pub fn is_supported_track_file(path: &Path) -> bool {
    trace!("Checking if {:?} is a music file", path);

    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("mp3" | "flac" | "ogg" | "opus" | "m4a" | "aac" | "wav" | "aiff")
    )
}
