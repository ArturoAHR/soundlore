use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn find_music_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_supported_music_file(e.path()))
        .map(|e| e.path().to_owned())
        .collect()
}

pub fn is_supported_music_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("mp3" | "flac" | "ogg" | "opus" | "m4a" | "aac" | "wav" | "aiff")
    )
}
