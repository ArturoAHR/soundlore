use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, DefaultStreamConfigError, OutputCallbackInfo, PauseStreamError,
    PlayStreamError, Stream,
};
use rtrb::{Consumer, PopError};
use thiserror::Error;
use tracing::error;

use crate::error::AppError;

#[derive(Debug, Error, Clone)]
pub enum PlaybackEngineError {
    #[error("output device not found")]
    OutputDeviceNotFound,
    #[error("default stream configuration error: {0}")]
    DefaultStreamConfiguration(#[from] DefaultStreamConfigError),
    #[error("build stream failed: {0}")]
    StreamBuildFailed(#[from] BuildStreamError),
    #[error("failed to pause stream: {0}")]
    StreamPauseFailed(#[from] PauseStreamError),
    #[error("failed to play stream: {0}")]
    StreamPlayFailed(#[from] PlayStreamError),
    #[error("stream hasn't been built")]
    MissingStream,
    #[error("failed to consume next samples: {0}")]
    EmptyBufferError(#[from] PopError),
}

pub struct PlaybackEngine {
    stream: Option<Stream>,
}

/*
 * TODO:
 * - Implement stream rebuild when default output device changes (poll default output device).
 */
impl PlaybackEngine {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn build_stream(
        &mut self,
        mut sample_buffer_consumer: Consumer<f32>,
    ) -> Result<(u32, u16), AppError> {
        let host = default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackEngineError::OutputDeviceNotFound)?;
        let config = device
            .default_output_config()
            .map_err(PlaybackEngineError::DefaultStreamConfiguration)?;

        let sample_rate = config.sample_rate();
        let channels = config.channels();

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    for slot in data.iter_mut() {
                        *slot = sample_buffer_consumer.pop().unwrap_or(0.0);
                    }
                },
                |error| error!("stream error: {error}"),
                None,
            )
            .map_err(PlaybackEngineError::StreamBuildFailed)?;

        self.stream = Some(stream);

        Ok((sample_rate, channels))
    }

    pub fn play_stream(&self) -> Result<(), AppError> {
        match &self.stream {
            Some(stream) => Ok(stream
                .play()
                .map_err(PlaybackEngineError::StreamPlayFailed)?),
            None => Err(PlaybackEngineError::MissingStream.into()),
        }
    }

    pub fn pause_stream(&self) -> Result<(), AppError> {
        match &self.stream {
            Some(stream) => Ok(stream
                .pause()
                .map_err(PlaybackEngineError::StreamPauseFailed)?),
            None => Err(PlaybackEngineError::MissingStream.into()),
        }
    }
}
