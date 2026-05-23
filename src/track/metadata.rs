use std::fs::File;
use std::path::Path;

use crate::error::AppError;
use crate::track::models::TrackProperties;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTag};
use symphonia::default::get_codecs;
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Clone, Error)]
pub enum TrackPropertiesReadError {
    #[error("missing file extension")]
    MissingExtension,

    #[error("no readable audio track")]
    MissingAudioTrack,

    #[error("missing codec parameters")]
    MissingCodecParameters,

    #[error("unknown codec")]
    UnknownCodec,

    #[error("missing sample rate")]
    MissingSampleRate,

    #[error("missing channel count")]
    MissingChannels,

    #[error("missing duration")]
    MissingDuration,
}

#[instrument]
pub fn read_track_metadata(path: &Path) -> Result<TrackProperties, AppError> {
    let file = File::open(path)?;
    let file_size_bytes = file.metadata()?.len() as i64;

    let file_path = path.to_string_lossy().to_string();
    let file_format = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| TrackPropertiesReadError::MissingExtension)?;

    let media_source_stream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension(&file_format);

    let mut format = symphonia::default::get_probe().probe(
        &hint,
        media_source_stream,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;

    let track = format
        .first_track_known_codec(TrackType::Audio)
        .ok_or_else(|| TrackPropertiesReadError::MissingAudioTrack)?;

    let Some(CodecParameters::Audio(codec_params)) = track.codec_params.as_ref() else {
        return Err(TrackPropertiesReadError::MissingCodecParameters.into());
    };

    let duration_secs = track
        .num_frames
        .zip(codec_params.sample_rate)
        .map(|(frames, rate)| frames as f64 / rate as f64)
        .ok_or_else(|| TrackPropertiesReadError::MissingDuration)?;

    let codec = get_codecs()
        .get_audio_decoder(codec_params.codec)
        .map(|decoder| decoder.codec.info.short_name.to_owned())
        .ok_or_else(|| TrackPropertiesReadError::UnknownCodec)?;

    let sample_rate = codec_params
        .sample_rate
        .map(|s| s as i64)
        .ok_or_else(|| TrackPropertiesReadError::MissingSampleRate)?;
    let channels = codec_params
        .channels
        .as_ref()
        .map(|c| c.count() as i64)
        .ok_or_else(|| TrackPropertiesReadError::MissingChannels)?;

    let bit_depth = codec_params.bits_per_sample.map(|b| b as i64);
    let bitrate_kbps = if duration_secs > 0.0 {
        Some(((file_size_bytes as f64 * 8.0) / duration_secs / 1000.0).round() as i64)
    } else {
        None
    };

    let mut track_properties = TrackProperties {
        file_path,
        file_size_bytes,
        file_format,

        codec,
        duration_secs,
        channels,
        sample_rate,
        bit_depth,
        bitrate_kbps,

        ..TrackProperties::default()
    };

    if let Some(revision) = format.metadata().current() {
        for tag in &revision.media.tags {
            if let Some(standard_tag) = &tag.std {
                match standard_tag {
                    StandardTag::TrackTitle(value) => {
                        track_properties.title = Some(value.to_string())
                    }
                    StandardTag::Artist(value) => track_properties.artist = Some(value.to_string()),
                    StandardTag::Album(value) => track_properties.album = Some(value.to_string()),
                    StandardTag::AlbumArtist(value) => {
                        track_properties.album_artist = Some(value.to_string());
                    }
                    StandardTag::TrackNumber(value) => {
                        track_properties.track_number = Some(*value as i64);
                    }
                    StandardTag::TrackTotal(value) => {
                        track_properties.track_total = Some(*value as i64);
                    }
                    StandardTag::DiscNumber(value) => {
                        track_properties.disc_number = Some(*value as i64);
                    }
                    StandardTag::DiscTotal(value) => {
                        track_properties.disc_total = Some(*value as i64);
                    }
                    StandardTag::RecordingYear(value) => {
                        track_properties.year = Some(*value as i64);
                    }
                    StandardTag::Genre(value) => track_properties.genre = Some(value.to_string()),
                    _ => {}
                }
            }

            let key = tag.raw.key.to_uppercase();
            match key.as_str() {
                "REPLAYGAIN_TRACK_GAIN" => {
                    track_properties.replaygain_track_gain_db = tag
                        .raw
                        .value
                        .to_string()
                        .trim_end_matches(" dB")
                        .parse()
                        .ok()
                }
                "REPLAYGAIN_TRACK_PEAK" => {
                    track_properties.replaygain_track_peak = tag.raw.value.to_string().parse().ok()
                }
                "REPLAYGAIN_ALBUM_GAIN" => {
                    track_properties.replaygain_album_gain_db = tag
                        .raw
                        .value
                        .to_string()
                        .trim_end_matches(" dB")
                        .parse()
                        .ok()
                }
                "REPLAYGAIN_ALBUM_PEAK" => {
                    track_properties.replaygain_album_peak = tag.raw.value.to_string().parse().ok()
                }
                _ => {}
            }
        }
    }

    Ok(track_properties)
}
