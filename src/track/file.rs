use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use crate::track::models::TrackProperties;
use crate::track::utils::fix_latin1_utf8_mojibake;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::well_known::{METADATA_ID_APEV2, METADATA_ID_ID3V1, METADATA_ID_ID3V2};
use symphonia::core::meta::{MetadataId, MetadataOptions, StandardTag, Tag};
use symphonia::default::get_codecs;
use thiserror::Error;
use tracing::{instrument, trace};

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

    #[error("unexpected error")]
    UnexpectedError,

    #[error("io error: {0}")]
    Io(Arc<std::io::Error>),

    #[error("symphonia error: {0}")]
    Symphonia(Arc<SymphoniaError>),
}

impl From<std::io::Error> for TrackPropertiesReadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(Arc::new(error))
    }
}

impl From<SymphoniaError> for TrackPropertiesReadError {
    fn from(error: SymphoniaError) -> Self {
        Self::Symphonia(Arc::new(error))
    }
}

#[instrument]
// TODO: Improve duration extraction by running background decode to ascertain exact sample count.
pub fn read_track_properties(path: &Path) -> Result<TrackProperties, TrackPropertiesReadError> {
    let file = File::open(path)?;
    let file_size_bytes = file.metadata()?.len() as i64;

    let file_path = path.to_string_lossy().to_string();
    let file_format = path
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .ok_or(TrackPropertiesReadError::MissingExtension)?;

    let media_source_stream =
        MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

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
        .ok_or(TrackPropertiesReadError::MissingAudioTrack)?;

    let Some(CodecParameters::Audio(codec_params)) = track.codec_params.as_ref() else {
        return Err(TrackPropertiesReadError::MissingCodecParameters);
    };
    let duration_secs = track
        .num_frames
        .zip(codec_params.sample_rate)
        .map(|(frames, rate)| frames as f64 / rate as f64)
        .ok_or(TrackPropertiesReadError::MissingDuration)?;

    // TODO: Get frame count from decoding if num_frames isn't populated.
    let frames = track
        .num_frames
        .ok_or(TrackPropertiesReadError::MissingDuration)? as i64;

    let codec = get_codecs()
        .get_audio_decoder(codec_params.codec)
        .map(|decoder| decoder.codec.info.short_name.to_owned())
        .ok_or(TrackPropertiesReadError::UnknownCodec)?;

    let sample_rate = codec_params
        .sample_rate
        .map(|s| s as i64)
        .ok_or(TrackPropertiesReadError::MissingSampleRate)?;
    let channels = codec_params
        .channels
        .as_ref()
        .map(|c| c.count() as i64)
        .ok_or(TrackPropertiesReadError::MissingChannels)?;

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
        frames,
        sample_rate,
        channels,
        bit_depth,
        bitrate_kbps,

        ..TrackProperties::default()
    };

    let mut revision_tags: Vec<(MetadataId, Vec<Tag>)> = Vec::new();

    // Extract Revision Tags here since cloning the whole Metadata Revision is expensive (it may have images)
    {
        let mut metadata = format.metadata();
        while metadata
            .current()
            .map(|revision| revision.info.metadata)
            .is_some()
        {
            if let Some(old_revision) = metadata.pop() {
                revision_tags.push((old_revision.info.metadata, old_revision.media.tags));
            } else {
                let latest_revision = metadata
                    .current()
                    .ok_or(TrackPropertiesReadError::UnexpectedError)?;

                revision_tags.push((
                    latest_revision.info.metadata,
                    latest_revision.media.tags.clone(),
                ));

                break;
            }
        }
    }

    // Sort the revision tags so that newer ones are applied last.
    revision_tags.sort_by_key(|(id, _)| match *id {
        METADATA_ID_ID3V1 => 0,
        METADATA_ID_APEV2 => 2,
        METADATA_ID_ID3V2 => 3,
        _ => 1, // Falls back to 1 (includes METADATA_ID_APEV1)
    });

    for (_, tags) in revision_tags {
        extract_revision_tags(&tags, &mut track_properties);
    }

    Ok(track_properties)
}

#[instrument(skip_all, fields(tag_count = tags.len()))]
fn extract_revision_tags(tags: &Vec<Tag>, track_properties: &mut TrackProperties) {
    for tag in tags {
        trace!(key = %tag.raw.key, std = ?tag.std, "Extracting tag");

        if let Some(standard_tag) = &tag.std {
            match standard_tag {
                StandardTag::TrackTitle(value) => {
                    track_properties.title = Some(fix_latin1_utf8_mojibake(value));
                }
                StandardTag::Artist(value) => {
                    track_properties.artist = Some(fix_latin1_utf8_mojibake(value));
                }
                StandardTag::Album(value) => {
                    track_properties.album = Some(fix_latin1_utf8_mojibake(value));
                }
                StandardTag::AlbumArtist(value) => {
                    track_properties.album_artist = Some(fix_latin1_utf8_mojibake(value));
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
                StandardTag::RecordingYear(value) if track_properties.year.is_none() => {
                    track_properties.year = Some(*value as i64);
                }
                StandardTag::RecordingDate(value) | StandardTag::ReleaseDate(value)
                    if track_properties.year.is_none() =>
                {
                    track_properties.year = value.get(..4).and_then(|s| s.parse().ok());
                }
                StandardTag::Genre(value) => {
                    track_properties.genre = Some(fix_latin1_utf8_mojibake(value));
                }
                StandardTag::ReplayGainTrackGain(value) => {
                    track_properties.replaygain_track_gain_db =
                        value.trim_end_matches(" dB").parse().ok();
                }
                StandardTag::ReplayGainTrackPeak(value) => {
                    track_properties.replaygain_track_peak = value.parse().ok();
                }
                StandardTag::ReplayGainAlbumGain(value) => {
                    track_properties.replaygain_album_gain_db =
                        value.trim_end_matches(" dB").parse().ok();
                }
                StandardTag::ReplayGainAlbumPeak(value) => {
                    track_properties.replaygain_album_peak = value.parse().ok();
                }
                _ => {}
            }
        }
    }
}
