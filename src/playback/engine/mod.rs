use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use cpal::{
    BuildStreamError, DefaultStreamConfigError, OutputCallbackInfo, PauseStreamError,
    PlayStreamError, Stream, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::{Consumer, PopError};
use thiserror::Error;
use tracing::error;

pub mod device;

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
        samples_played: Arc<AtomicU64>,
        samples_to_skip: Arc<AtomicU64>,
    ) -> Result<(u32, u16), PlaybackEngineError>;
    fn play_stream(&self) -> Result<(), PlaybackEngineError>;
    fn pause_stream(&self) -> Result<(), PlaybackEngineError>;
}

pub enum PlaybackEngineStatus {
    Playing,
    Paused,
}

pub struct AudioEngine {
    stream: Option<Stream>,

    status: PlaybackEngineStatus,
}

/*
 * TODO:
 * - Implement stream rebuild when default output device changes (poll default output device).
 */
impl AudioEngine {
    pub fn new() -> Self {
        Self {
            stream: None,
            status: PlaybackEngineStatus::Playing,
        }
    }
}

impl PlaybackEngine for AudioEngine {
    fn build_stream(
        &mut self,
        mut sample_buffer_consumer: Consumer<f32>,
        samples_played: Arc<AtomicU64>,
        samples_to_skip: Arc<AtomicU64>,
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
                let mut skipped_samples_left = samples_to_skip.load(Ordering::Relaxed);
                let mut skipped_samples = 0;

                while skipped_samples_left > 0 && sample_buffer_consumer.pop().is_ok() {
                    skipped_samples_left -= 1;
                    skipped_samples += 1;
                }

                if skipped_samples > 0 {
                    samples_to_skip.fetch_sub(skipped_samples, Ordering::Relaxed);
                }

                if skipped_samples_left > 0 {
                    for slot in data.iter_mut() {
                        *slot = 0.0;
                    }

                    return;
                }

                for slot in data.iter_mut() {
                    let Ok(sample) = sample_buffer_consumer.pop() else {
                        *slot = 0.0;
                        continue;
                    };

                    *slot = sample;
                    samples_played.fetch_add(1, Ordering::Relaxed);
                }
            },
            |error| error!("stream error: {error}"),
            None,
        )?;

        self.stream = Some(stream);

        Ok((sample_rate, channels))
    }

    fn play_stream(&self) -> Result<(), PlaybackEngineError> {
        let Some(stream) = self.stream.as_ref() else {
            return Err(PlaybackEngineError::MissingStream);
        };

        if matches!(self.status, PlaybackEngineStatus::Paused) {
            stream.play()?
        }

        Ok(())
    }

    fn pause_stream(&self) -> Result<(), PlaybackEngineError> {
        let Some(stream) = self.stream.as_ref() else {
            return Err(PlaybackEngineError::MissingStream);
        };

        if matches!(self.status, PlaybackEngineStatus::Playing) {
            stream.pause()?
        }

        Ok(())
    }
}
