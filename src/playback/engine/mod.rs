use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
    },
};

use cpal::{
    BuildStreamError, DefaultStreamConfigError, PauseStreamError, PlayStreamError, SampleFormat,
    Stream, default_host,
    traits::{DeviceTrait, HostTrait},
};
use rtrb::{Consumer, PopError};
use thiserror::Error;
use tracing::instrument;

use crate::playback::{
    GenerationCounter,
    engine::thread::{AudioEngineStreamBuildArguments, build_output_stream},
};

pub mod constants;
pub mod device;
pub mod thread;

#[derive(Debug, Error, Clone)]
pub enum PlaybackEngineError {
    #[error("output device not found")]
    OutputDeviceNotFound,

    #[error("default stream configuration error: {0}")]
    DefaultStreamConfiguration(Arc<DefaultStreamConfigError>),

    #[error("build stream failed: {0}")]
    StreamBuildFailed(Arc<BuildStreamError>),

    #[error("failed to pause stream: {0}")]
    StreamPauseFailed(Arc<PauseStreamError>),

    #[error("failed to play stream: {0}")]
    StreamPlayFailed(Arc<PlayStreamError>),

    #[error("stream hasn't been built")]
    MissingStream,

    #[error("failed to consume next samples: {0}")]
    EmptyBufferError(Arc<PopError>),

    #[error("unsupported output sample format")]
    UnsupportedSampleFormat,
}

impl From<DefaultStreamConfigError> for PlaybackEngineError {
    fn from(error: DefaultStreamConfigError) -> Self {
        Self::DefaultStreamConfiguration(Arc::new(error))
    }
}

impl From<BuildStreamError> for PlaybackEngineError {
    fn from(error: BuildStreamError) -> Self {
        Self::StreamBuildFailed(Arc::new(error))
    }
}

impl From<PauseStreamError> for PlaybackEngineError {
    fn from(error: PauseStreamError) -> Self {
        Self::StreamPauseFailed(Arc::new(error))
    }
}

impl From<PlayStreamError> for PlaybackEngineError {
    fn from(error: PlayStreamError) -> Self {
        Self::StreamPlayFailed(Arc::new(error))
    }
}

impl From<PopError> for PlaybackEngineError {
    fn from(error: PopError) -> Self {
        Self::EmptyBufferError(Arc::new(error))
    }
}

pub trait PlaybackEngine {
    fn build_stream(
        &mut self,
        sample_buffer_consumer: Consumer<f32>,
        samples_played: Arc<AtomicU64>,
        track_start_timestamp: Arc<AtomicI64>,
        samples_played_timestamp_offset: Arc<AtomicU64>,
        generation_counter: Arc<GenerationCounter>,
    ) -> Result<(u32, u16), PlaybackEngineError>;
    fn pause(&mut self) -> Result<(), PlaybackEngineError>;
    fn play(&mut self) -> Result<(), PlaybackEngineError>;
    fn status(&self) -> &PlaybackEngineStatus;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackEngineStatus {
    Playing,
    Paused,
}

pub struct AudioEngine {
    stream: Option<Stream>,

    paused: Arc<AtomicBool>,
    status: PlaybackEngineStatus,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            stream: None,

            paused: Arc::new(AtomicBool::new(true)),
            status: PlaybackEngineStatus::Paused,
        }
    }

    #[instrument(skip_all, ret, level = "debug", fields(current_status = ?self.status, status = ?status))]
    fn set_status(&mut self, status: PlaybackEngineStatus) {
        if self.status == status {
            return;
        }

        self.status = status;

        self.paused.store(
            matches!(self.status, PlaybackEngineStatus::Paused),
            Ordering::Relaxed,
        );
    }
}

impl PlaybackEngine for AudioEngine {
    #[instrument(skip_all, err, level = "debug")]
    fn build_stream(
        &mut self,
        sample_buffer_consumer: Consumer<f32>,
        samples_played: Arc<AtomicU64>,
        track_start_timestamp: Arc<AtomicI64>,
        samples_played_timestamp_offset: Arc<AtomicU64>,
        generation_counter: Arc<GenerationCounter>,
    ) -> Result<(u32, u16), PlaybackEngineError> {
        let host = default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackEngineError::OutputDeviceNotFound)?;
        let config = device.default_output_config()?;

        let sample_rate = config.sample_rate();
        let channels = config.channels();

        let build_arguments = AudioEngineStreamBuildArguments {
            sample_buffer_consumer,
            config: config.clone().into(),
            device,
            samples_played,
            track_start_timestamp,
            samples_played_timestamp_offset,
            generation_counter,
            paused: Arc::clone(&self.paused),
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => build_output_stream::<f32>(build_arguments)?,
            SampleFormat::I16 => build_output_stream::<i16>(build_arguments)?,
            SampleFormat::U16 => build_output_stream::<u16>(build_arguments)?,

            _ => return Err(PlaybackEngineError::UnsupportedSampleFormat),
        };

        self.stream = Some(stream);

        Ok((sample_rate, channels))
    }

    #[instrument(skip_all, err, level = "debug")]
    fn pause(&mut self) -> Result<(), PlaybackEngineError> {
        if self.stream.is_none() {
            return Err(PlaybackEngineError::MissingStream);
        }

        self.set_status(PlaybackEngineStatus::Paused);

        Ok(())
    }

    #[instrument(skip_all, err, level = "debug")]
    fn play(&mut self) -> Result<(), PlaybackEngineError> {
        if self.stream.is_none() {
            return Err(PlaybackEngineError::MissingStream);
        }

        self.set_status(PlaybackEngineStatus::Playing);

        Ok(())
    }

    fn status(&self) -> &PlaybackEngineStatus {
        &self.status
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}
