use std::path::{Path, PathBuf};

use tracing::{debug, instrument, trace};
use walkdir::WalkDir;

#[instrument(level = "debug")]
pub fn find_track_files(root: &Path) -> Vec<PathBuf> {
    debug!("Finding music files in {:?}", root);

    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_supported_track_file(e.path()))
        .map(|e| e.path().to_owned())
        .collect()
}

#[instrument(level = "trace")]
pub fn is_supported_track_file(path: &Path) -> bool {
    trace!("Checking if {:?} is a music file", path);

    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("mp3" | "flac" | "ogg" | "m4a" | "aac" | "wav" | "aiff")
    )
}

pub fn get_application_directory_name() -> String {
    if cfg!(debug_assertions) {
        "soundlore-dev".to_owned()
    } else {
        "soundlore".to_owned()
    }
}
