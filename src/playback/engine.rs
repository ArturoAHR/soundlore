use std::{fmt::Debug, sync::Arc};

use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, DefaultStreamConfigError, OutputCallbackInfo, PauseStreamError,
    PlayStreamError, Stream,
};
use rtrb::{Consumer, PopError};
use thiserror::Error;
use tracing::error;

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
    ) -> Result<(u32, u16), PlaybackEngineError>;
    fn play_stream(&self) -> Result<(), PlaybackEngineError>;
    fn pause_stream(&self) -> Result<(), PlaybackEngineError>;
}

pub struct AudioEngine {
    stream: Option<Stream>,
}

/*
 * TODO:
 * - Implement stream rebuild when default output device changes (poll default output device).
 */
impl AudioEngine {
    pub fn new() -> Self {
        Self { stream: None }
    }
}

impl PlaybackEngine for AudioEngine {
    fn build_stream(
        &mut self,
        mut sample_buffer_consumer: Consumer<f32>,
    ) -> Result<(u32, u16), PlaybackEngineError> {
        let host = default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackEngineError::OutputDeviceNotFound)?;
        let config = device.default_output_config()?;

        let sample_rate = config.sample_rate();
        let channels = config.channels();

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &OutputCallbackInfo| {
                for slot in data.iter_mut() {
                    *slot = sample_buffer_consumer.pop().unwrap_or(0.0);
                }
            },
            |error| error!("stream error: {error}"),
            None,
        )?;

        self.stream = Some(stream);

        Ok((sample_rate, channels))
    }

    fn play_stream(&self) -> Result<(), PlaybackEngineError> {
        match &self.stream {
            Some(stream) => Ok(stream.play()?),
            None => Err(PlaybackEngineError::MissingStream),
        }
    }

    fn pause_stream(&self) -> Result<(), PlaybackEngineError> {
        match &self.stream {
            Some(stream) => Ok(stream.pause()?),
            None => Err(PlaybackEngineError::MissingStream),
        }
    }
}
