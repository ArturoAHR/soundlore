use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

use cpal::{
    BuildStreamError, DefaultStreamConfigError, Device, FromSample, OutputCallbackInfo,
    PauseStreamError, PlayStreamError, SampleFormat, SizedSample, Stream, StreamConfig,
    default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::{Consumer, PopError};
use thiserror::Error;
use tracing::error;

use crate::playback::GenerationCounter;

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

        let stream = match config.sample_format() {
            SampleFormat::F32 => build_output_stream::<f32>(
                device,
                &config.into(),
                sample_buffer_consumer,
                samples_played,
                track_start_timestamp,
                samples_played_timestamp_offset,
                generation_counter,
            )?,
            SampleFormat::I16 => build_output_stream::<i16>(
                device,
                &config.into(),
                sample_buffer_consumer,
                samples_played,
                track_start_timestamp,
                samples_played_timestamp_offset,
                generation_counter,
            )?,
            SampleFormat::U16 => build_output_stream::<u16>(
                device,
                &config.into(),
                sample_buffer_consumer,
                samples_played,
                track_start_timestamp,
                samples_played_timestamp_offset,
                generation_counter,
            )?,

            _ => return Err(PlaybackEngineError::UnsupportedSampleFormat),
        };

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

fn build_output_stream<T>(
    device: Device,
    config: &StreamConfig,
    mut sample_buffer_consumer: Consumer<f32>,
    samples_played: Arc<AtomicU64>,
    track_start_timestamp: Arc<AtomicI64>,
    samples_played_timestamp_offset: Arc<AtomicU64>,
    generation_counter: Arc<GenerationCounter>,
) -> Result<Stream, PlaybackEngineError>
where
    T: SizedSample + FromSample<f32>,
{
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &OutputCallbackInfo| {
            let audio_pipeline_generation =
                generation_counter.audio_pipeline.load(Ordering::Acquire);
            let audio_engine_generation = generation_counter.audio_engine.load(Ordering::Relaxed);

            if audio_engine_generation != audio_pipeline_generation {
                let start_timestamp = samples_played.load(Ordering::Relaxed) as i64
                    - samples_played_timestamp_offset.load(Ordering::Relaxed) as i64;

                track_start_timestamp.store(start_timestamp, Ordering::Release);

                // Clear ring buffer
                while sample_buffer_consumer.pop().is_ok() {}

                generation_counter
                    .audio_engine
                    .store(audio_pipeline_generation, Ordering::Release);
            }

            let mut played = 0;
            for slot in data.iter_mut() {
                let Ok(sample) = sample_buffer_consumer.pop() else {
                    *slot = T::from_sample_(0.0);
                    continue;
                };

                *slot = T::from_sample_(sample);
                played += 1;
            }

            samples_played.fetch_add(played, Ordering::Relaxed);
        },
        |error| error!("stream error: {error}"),
        None,
    )?;

    Ok(stream)
}
