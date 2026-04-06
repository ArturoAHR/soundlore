use std::path::Path;

use std::fs::File;

use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;

use crate::error::AppError;
use crate::track::models::TrackMetadata;

pub fn read_track_metadata(path: &Path) -> Result<TrackMetadata, AppError> {
    let track_file = File::open(path)?;
    let track_file_size = track_file.metadata()?.len() as i64;
    let mss = MediaSourceStream::new(Box::new(track_file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    let mut track_metadata = TrackMetadata {
        file_path: path.to_string_lossy().to_string(),
        file_size: Some(track_file_size),
        format: path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase()),
        ..Default::default()
    };

    (
        track_metadata.duration_secs,
        track_metadata.bitrate,
        track_metadata.sample_rate,
        track_metadata.channels,
    ) = if let Some(track) = format.tracks().first() {
        let params = &track.codec_params;
        let duration = params
            .n_frames
            .zip(params.sample_rate)
            .map(|(frames, rate)| frames as f64 / rate as f64);
        (
            duration,
            params.bits_per_sample.map(|b| b as i64),
            params.sample_rate.map(|s| s as i64),
            params.channels.map(|c| c.count() as i64),
        )
    } else {
        (None, None, None, None)
    };

    let mut container_metadata = probed.metadata;
    let container_revision = container_metadata.get().and_then(|m| m.current().cloned());
    let format_revision = format.metadata().current().cloned();

    for revision in [container_revision, format_revision].into_iter().flatten() {
        for tag in revision.tags() {
            if let Some(std_key) = tag.std_key {
                match std_key {
                    StandardTagKey::TrackTitle => {
                        track_metadata.title = Some(tag.value.to_string())
                    }
                    StandardTagKey::Artist => track_metadata.artist = Some(tag.value.to_string()),
                    StandardTagKey::Album => track_metadata.album = Some(tag.value.to_string()),
                    StandardTagKey::AlbumArtist => {
                        track_metadata.album_artist = Some(tag.value.to_string())
                    }
                    StandardTagKey::TrackNumber => {
                        track_metadata.track_number = tag.value.to_string().parse().ok()
                    }
                    StandardTagKey::DiscNumber => {
                        track_metadata.disc_number = tag.value.to_string().parse().ok()
                    }
                    StandardTagKey::Date => {
                        track_metadata.year = tag.value.to_string().parse().ok()
                    }
                    StandardTagKey::Genre => track_metadata.genre = Some(tag.value.to_string()),
                    _ => {}
                }
            }
        }
    }

    Ok(track_metadata)
}
