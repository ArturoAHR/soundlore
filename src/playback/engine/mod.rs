use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
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
use tracing::{error, instrument};

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
    fn play_stream(&mut self) -> Result<(), PlaybackEngineError>;
    fn pause_stream(&mut self) -> Result<(), PlaybackEngineError>;
    fn status(&self) -> &PlaybackEngineStatus;
}

#[derive(PartialEq, Clone, Debug)]
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

            paused: Arc::new(AtomicBool::new(false)),

            status: PlaybackEngineStatus::Playing,
        }
    }

    #[instrument(skip(self), ret, level = "trace")]
    fn set_status(&mut self, status: PlaybackEngineStatus) {
        if self.status == status {
            return;
        }

        self.status = status;

        self.paused.store(
            self.status == PlaybackEngineStatus::Paused,
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
            device,
            samples_played,
            track_start_timestamp,
            samples_played_timestamp_offset,
            generation_counter,
            paused: Arc::clone(&self.paused),
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                build_output_stream::<f32>(sample_buffer_consumer, &config.into(), build_arguments)?
            }
            SampleFormat::I16 => {
                build_output_stream::<i16>(sample_buffer_consumer, &config.into(), build_arguments)?
            }
            SampleFormat::U16 => {
                build_output_stream::<u16>(sample_buffer_consumer, &config.into(), build_arguments)?
            }

            _ => return Err(PlaybackEngineError::UnsupportedSampleFormat),
        };

        self.stream = Some(stream);

        self.pause_stream()?;

        Ok((sample_rate, channels))
    }

    #[instrument(skip_all, err, level = "debug")]
    fn play_stream(&mut self) -> Result<(), PlaybackEngineError> {
        let Some(stream) = self.stream.as_ref() else {
            return Err(PlaybackEngineError::MissingStream);
        };

        if self.status == PlaybackEngineStatus::Paused {
            stream.play()?;

            self.set_status(PlaybackEngineStatus::Playing);
        }

        Ok(())
    }

    #[instrument(skip_all, err, level = "debug")]
    fn pause_stream(&mut self) -> Result<(), PlaybackEngineError> {
        let Some(stream) = self.stream.as_ref() else {
            return Err(PlaybackEngineError::MissingStream);
        };

        if self.status == PlaybackEngineStatus::Playing {
            stream.pause()?;

            self.set_status(PlaybackEngineStatus::Paused);
        }

        Ok(())
    }

    fn status(&self) -> &PlaybackEngineStatus {
        &self.status
    }
}

struct AudioEngineStreamBuildArguments {
    pub device: Device,
    pub samples_played: Arc<AtomicU64>,
    pub track_start_timestamp: Arc<AtomicI64>,
    pub samples_played_timestamp_offset: Arc<AtomicU64>,
    pub generation_counter: Arc<GenerationCounter>,
    pub paused: Arc<AtomicBool>,
}

fn build_output_stream<T>(
    mut sample_buffer_consumer: Consumer<f32>,
    config: &StreamConfig,
    arguments: AudioEngineStreamBuildArguments,
) -> Result<Stream, PlaybackEngineError>
where
    T: SizedSample + FromSample<f32>,
{
    let device = arguments.device;
    let samples_played = arguments.samples_played;
    let track_start_timestamp = arguments.track_start_timestamp;
    let samples_played_timestamp_offset = arguments.samples_played_timestamp_offset;
    let generation_counter = arguments.generation_counter;
    let paused = arguments.paused;

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
                if !paused.load(Ordering::Relaxed) {
                    if let Ok(sample) = sample_buffer_consumer.pop() {
                        *slot = T::from_sample_(sample);
                        played += 1;

                        continue;
                    }
                }

                *slot = T::from_sample_(0.0);
            }

            samples_played.fetch_add(played, Ordering::Relaxed);
        },
        |error| error!("stream error: {error}"),
        None,
    )?;

    Ok(stream)
}
