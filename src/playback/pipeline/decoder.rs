use std::{fs::File, path::PathBuf, sync::Arc};

use symphonia::{
    core::{
        codecs::{
            audio::{AudioDecoder as SymphoniaDecoder, AudioDecoderOptions},
            CodecParameters,
        },
        errors::Error as SymphoniaError,
        formats::{probe::Hint, FormatOptions, FormatReader, TrackType},
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default::get_codecs,
};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum AudioDecoderError {
    #[error("unexpected error")]
    UnexpectedError,

    #[error("missing audio track")]
    MissingAudioTrack,

    #[error("missing codec parameters")]
    MissingCodecParameters,

    #[error("recoverable audio decoder error: {0}")]
    RecoverableDecoderError(String),

    #[error("audio decoder error: {0}")]
    DecoderError(Arc<SymphoniaError>),

    #[error("io error: {0}")]
    Io(Arc<std::io::Error>),
}

impl From<std::io::Error> for AudioDecoderError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(Arc::new(error))
    }
}

impl From<SymphoniaError> for AudioDecoderError {
    fn from(error: SymphoniaError) -> Self {
        Self::DecoderError(Arc::new(error))
    }
}

pub struct AudioDecoder {
    pub track: AudioDecoderTrack,
    pub status: AudioDecoderStatus,

    pub delivered_frames: u64,

    demuxer: Box<dyn FormatReader>,
    decoder: Box<dyn SymphoniaDecoder>,
}

pub enum AudioDecoderStatus {
    Finished,
    Decoding,
}

pub struct AudioDecoderTrack {
    pub path: PathBuf,
    packet_track_id: u32,

    pub sample_rate: u32,
    pub channels: u16,

    pub total_frames: u64,
}

impl AudioDecoder {
    pub fn build(track_path: PathBuf) -> Result<Self, AudioDecoderError> {
        let track_file = File::open(&track_path)?;

        let media_source_stream = MediaSourceStream::new(Box::new(track_file), Default::default());

        let mut hint = Hint::new();

        if let Some(file_format) = track_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
        {
            hint.with_extension(&file_format);
        }

        let demuxer = symphonia::default::get_probe().probe(
            &hint,
            media_source_stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        let track = demuxer
            .as_ref()
            .first_track_known_codec(TrackType::Audio)
            .ok_or_else(|| AudioDecoderError::MissingAudioTrack)?;

        let Some(CodecParameters::Audio(codec_parameters)) = track.codec_params.as_ref() else {
            return Err(AudioDecoderError::MissingCodecParameters);
        };

        let decoder =
            get_codecs().make_audio_decoder(&codec_parameters, &AudioDecoderOptions::default())?;

        let (Some(sample_rate), Some(channels), Some(total_frames)) = (
            codec_parameters.sample_rate,
            codec_parameters
                .channels
                .as_ref()
                .and_then(|channels| Some(channels.count())),
            track.num_frames,
        ) else {
            return Err(AudioDecoderError::MissingCodecParameters);
        };

        Ok(Self {
            track: AudioDecoderTrack {
                path: track_path,
                packet_track_id: track.id,
                sample_rate,
                channels: channels as u16,
                total_frames,
            },
            delivered_frames: 0,
            status: AudioDecoderStatus::Decoding,
            demuxer,
            decoder,
        })
    }

    /// Returns decoded samples, if there are no more samples `None` will be returned.
    pub fn decode(&mut self) -> Result<Option<Vec<f32>>, AudioDecoderError> {
        let mut packet = None;

        while let Ok(Some(current_packet)) = self.demuxer.next_packet() {
            if current_packet.track_id == self.track.packet_track_id {
                packet = Some(current_packet);

                break;
            }
        }

        let Some(packet) = packet else {
            // No more packets to decode.

            self.status = AudioDecoderStatus::Finished;

            return Ok(None);
        };

        let generic_audio_buffer = match self.decoder.decode(&packet) {
            Ok(generic_audio_buffer) => Ok(generic_audio_buffer),
            Err(decode_error) => {
                match decode_error {
                    // Recoverable decoder errors
                    SymphoniaError::ResetRequired => {
                        self.decoder.reset();

                        Err(AudioDecoderError::RecoverableDecoderError(
                            "reset was required".to_owned(),
                        ))
                    }
                    SymphoniaError::DecodeError(error) => Err(
                        AudioDecoderError::RecoverableDecoderError(error.to_string()),
                    ),
                    SymphoniaError::IoError(error) => Err(
                        AudioDecoderError::RecoverableDecoderError(error.to_string()),
                    ),
                    // Unrecoverable decoder errors
                    error => Err(AudioDecoderError::DecoderError(Arc::new(error))),
                }
            }
        }?;

        let mut samples: Vec<f32> = Vec::new();

        generic_audio_buffer.copy_to_vec_interleaved(&mut samples);

        self.delivered_frames += samples.len() as u64;

        Ok(Some(samples))
    }
}
