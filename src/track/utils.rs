use std::str::from_utf8;

use tracing::instrument;

use crate::track::models::Track;

#[instrument(level = "trace", ret(level = "trace"))]
pub fn fix_latin1_utf8_mojibake(s: &str) -> String {
    // Check if the character code is within ASCII
    if !s.chars().all(|c| (c as u32) < 0x100) {
        return s.to_owned();
    }

    let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();

    match from_utf8(&bytes) {
        // Skip non-trivial UTF-8 reinterpretations multiple byte sequences
        Ok(decoded) if decoded.chars().any(|c| (c as u32) >= 0x80) => decoded.to_owned(),

        _ => s.to_owned(),
    }
}

pub fn get_track_name(track: &Track) -> String {
    format!(
        "{} - {}",
        track.artist.clone().unwrap_or("Unknown".to_owned()),
        track.title.clone().unwrap_or("Untitled".to_owned())
    )
}
