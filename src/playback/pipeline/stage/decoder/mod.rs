use std::{fs::File, sync::Arc};

use symphonia::{
    core::{
        codecs::{
            CodecParameters,
            audio::{AudioDecoder as SymphoniaDecoder, AudioDecoderOptions},
        },
        errors::Error as SymphoniaError,
        formats::{
            FormatOptions, FormatReader, SeekMode, SeekTo, SeekedTo, TrackType, probe::Hint,
        },
        io::MediaSourceStream,
        meta::MetadataOptions,
        units::Timestamp,
    },
    default::get_codecs,
};
use thiserror::Error;

use crate::{
    playback::pipeline::{stage::AudioPipelineSamples, thread::AudioPipelineThreadEvent},
    track::models::Track,
};

pub mod stage;

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
    pub status: AudioDecoderStatus,
    pub pending_events: Vec<AudioPipelineThreadEvent>,

    packet_track_id: u32,

    demuxer: Box<dyn FormatReader>,
    decoder: Box<dyn SymphoniaDecoder>,
}

pub enum AudioDecoderStatus {
    Finished,
    Decoding,
}

impl AudioDecoder {
    pub fn build(track: &Track) -> Result<Self, AudioDecoderError> {
        let track_file = File::open(&track.file_path)?;

        let media_source_stream = MediaSourceStream::new(Box::new(track_file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension(&track.file_format);

        let demuxer = symphonia::default::get_probe().probe(
            &hint,
            media_source_stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        let audio_track = demuxer
            .as_ref()
            .first_track_known_codec(TrackType::Audio)
            .ok_or_else(|| AudioDecoderError::MissingAudioTrack)?;

        let Some(CodecParameters::Audio(codec_parameters)) = audio_track.codec_params.as_ref()
        else {
            return Err(AudioDecoderError::MissingCodecParameters);
        };

        let decoder =
            get_codecs().make_audio_decoder(&codec_parameters, &AudioDecoderOptions::default())?;

        Ok(Self {
            pending_events: Vec::new(),

            packet_track_id: audio_track.id,
            status: AudioDecoderStatus::Decoding,
            demuxer,
            decoder,
        })
    }

    /// Returns decoded samples, if there are no more samples `None` will be returned.
    pub fn decode(&mut self) -> Result<AudioPipelineSamples, AudioDecoderError> {
        let mut packet = None;

        while let Ok(Some(current_packet)) = self.demuxer.next_packet() {
            if current_packet.track_id == self.packet_track_id {
                packet = Some(current_packet);

                break;
            }
        }

        let Some(packet) = packet else {
            // No more packets to decode.

            self.status = AudioDecoderStatus::Finished;
            self.pending_events
                .push(AudioPipelineThreadEvent::DecodingFinished);

            return Ok(AudioPipelineSamples::End(vec![]));
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

        Ok(AudioPipelineSamples::Chunk(samples))
    }

    pub fn seek(
        &mut self,
        timestamp: u64,
        seek_mode: SeekMode,
    ) -> Result<SeekedTo, AudioDecoderError> {
        let seeked_to = self.demuxer.seek(
            seek_mode,
            SeekTo::Timestamp {
                ts: Timestamp::new(timestamp as i64),
                track_id: self.packet_track_id,
            },
        )?;

        self.decoder.reset();

        self.status = AudioDecoderStatus::Decoding;

        Ok(seeked_to)
    }
}
