use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use thiserror::Error;
use tracing::{error, instrument, warn};

use crate::{
    playback::{
        GenerationCounter,
        constants::SAMPLE_BUFFER_CAPACITY,
        pipeline::{
            command::{AudioPipelineCommandReceiver, AudioPipelineCommandReceiverError},
            config::AudioPipelineConfiguration,
            sink::{AudioSink, AudioSinkError},
            stage::{
                channel_converter::AudioChannelConverterError, decoder::AudioDecoderError,
                resampler::AudioResamplerError,
            },
            thread::{
                AudioPipelineProcessDirective, AudioPipelineThreadCommand, AudioPipelineThreadEvent,
            },
            track::{AudioTrackPipeline, AudioTrackPipelineStatus},
        },
    },
    track::models::Track,
};

pub mod builder;
pub mod command;
pub mod config;
pub mod event;
pub mod sink;
pub mod stage;
pub mod thread;
pub mod track;

#[derive(Debug, Error, Clone)]
pub enum AudioPipelineError {
    #[error("there are no more samples to decode for current track.")]
    AudioTrackPipelineFinished,

    #[error("audio pipeline command receiver error: {0}")]
    AudioPipelineCommandReceiver(#[from] AudioPipelineCommandReceiverError),

    #[error("decoder error: {0}")]
    Decoder(#[from] AudioDecoderError),

    #[error("channel converter error: {0}")]
    ChannelConverter(#[from] AudioChannelConverterError),

    #[error("resampler error: {0}")]
    Resampler(#[from] AudioResamplerError),

    #[error("sink error: {0}")]
    Sink(#[from] AudioSinkError),
}

pub struct AudioPipeline {
    audio_track_pipelines: Vec<AudioTrackPipeline>,
    audio_sink: AudioSink,

    command_receiver: AudioPipelineCommandReceiver,
    pub status: AudioPipelineStatus,

    configuration: AudioPipelineConfiguration,

    samples_played_timestamp_offset: Arc<AtomicU64>,
    generation_counter: Arc<GenerationCounter>,
}

#[derive(PartialEq)]
pub enum AudioPipelineStatus {
    Idle,
    Active,
    Paused,
}

#[derive(Debug)]
pub enum AudioPipelineCommand {
    // ChangeOutputFormat(AudioFormat),
    Seek(u64),
    Stop,
}

#[derive(Debug, Clone)]
pub enum AudioPipelineCommandOutcome {
    SeekedTo(u64),
}

#[derive(Debug, Clone)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioPipeline {
    pub fn new(
        configuration: AudioPipelineConfiguration,
        audio_sink: AudioSink,
        command_receiver: AudioPipelineCommandReceiver,
        track: Option<Track>,
        samples_played_timestamp_offset: Arc<AtomicU64>,
        generation_counter: Arc<GenerationCounter>,
    ) -> Self {
        let mut audio_pipeline = Self {
            audio_sink,
            command_receiver,
            configuration,
            audio_track_pipelines: Vec::new(),
            status: AudioPipelineStatus::Idle,

            samples_played_timestamp_offset,
            generation_counter,
        };

        if let Some(error) = track.and_then(|track| audio_pipeline.play_track(track).err()) {
            error!("Failed to create audio track pipeline when creating audio pipeline {error}");
        }

        audio_pipeline
    }

    #[instrument(skip_all)]
    pub fn play_track(&mut self, track: Track) -> Result<(), AudioPipelineError> {
        let track_id = track.id.clone();

        let audio_track_pipeline = AudioTrackPipeline::build(track, self.configuration.clone())?;

        self.audio_track_pipelines = vec![audio_track_pipeline];

        self.configuration
            .event_emitter
            .emit(AudioPipelineThreadEvent::ActiveTrackChanged { track_id });

        self.set_status(AudioPipelineStatus::Active);

        self.increase_generation_counter(0);

        Ok(())
    }

    #[instrument(skip_all)]
    pub fn resume(&mut self) -> Result<(), AudioPipelineError> {
        let Some(audio_track_pipeline) = self.audio_track_pipelines.get_mut(0) else {
            warn!("Attempted to resume playback without a track to play.");

            return Ok(());
        };

        match audio_track_pipeline.status {
            AudioTrackPipelineStatus::ProducingSamples | AudioTrackPipelineStatus::Ready => {
                self.set_status(AudioPipelineStatus::Active);
            }
            AudioTrackPipelineStatus::Finished => {
                if self.audio_sink.is_empty()
                    && self.audio_sink.is_engine_buffer_empty()
                    && self.status == AudioPipelineStatus::Idle
                {
                    let track = audio_track_pipeline.configuration.track.clone();

                    self.play_track(track)?;
                } else {
                    self.set_status(AudioPipelineStatus::Active);
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn handle_command(
        &mut self,
        command: AudioPipelineThreadCommand,
    ) -> Result<Option<AudioPipelineProcessDirective>, AudioPipelineError> {
        let mut audio_pipeline_command = None;
        match command {
            AudioPipelineThreadCommand::Play(track) => {
                self.play_track(track)?;
            }
            AudioPipelineThreadCommand::Pause => {
                self.set_status(AudioPipelineStatus::Paused);
            }
            AudioPipelineThreadCommand::Resume => {
                self.resume()?;
            }
            AudioPipelineThreadCommand::Stop => {
                self.set_status(AudioPipelineStatus::Idle);

                audio_pipeline_command = Some(AudioPipelineCommand::Stop);
            }
            AudioPipelineThreadCommand::PlayNext => {
                warn!("Play Next command has been issued but it's not implemented yet.");
            }
            AudioPipelineThreadCommand::PlayPrevious => {
                warn!("Play Previous command has been issued but it's not implemented yet.");
            }
            AudioPipelineThreadCommand::Seek(seek_timestamp) => {
                audio_pipeline_command = Some(AudioPipelineCommand::Seek(seek_timestamp));
            }
            AudioPipelineThreadCommand::ChangeNextTrack(_) => {
                warn!("Change Next Track command has been issued but it's not implemented yet.");
            }
            AudioPipelineThreadCommand::ChangeOutput {
                output,
                audio_engine_producer,
            } => {
                self.audio_sink =
                    AudioSink::new(audio_engine_producer, Arc::clone(&self.generation_counter));

                self.configuration.output = output;
            }
            AudioPipelineThreadCommand::Exit => {
                return Ok(Some(AudioPipelineProcessDirective::Exit));
            }
        }

        let mut outcomes = Vec::new();
        if let (Some(audio_track_pipeline), Some(audio_pipeline_command)) = (
            self.audio_track_pipelines.get_mut(0),
            audio_pipeline_command,
        ) {
            outcomes.extend(audio_track_pipeline.handle_command(&audio_pipeline_command)?);
        }

        for outcome in outcomes {
            match outcome {
                AudioPipelineCommandOutcome::SeekedTo(new_decoder_timestamp) => {
                    self.increase_generation_counter(new_decoder_timestamp);
                }
            }
        }

        Ok(None)
    }

    pub fn set_status(&mut self, status: AudioPipelineStatus) {
        if self.status == status {
            return;
        }

        self.status = status;

        match self.status {
            AudioPipelineStatus::Idle | AudioPipelineStatus::Paused => {
                self.configuration
                    .event_emitter
                    .emit(AudioPipelineThreadEvent::StoppedAudioPipeline);
            }
            AudioPipelineStatus::Active => {
                self.configuration
                    .event_emitter
                    .emit(AudioPipelineThreadEvent::StartedAudioPipeline);
            }
        }
    }

    fn increase_generation_counter(&mut self, decoder_timestamp: u64) {
        self.audio_sink.clear();

        let timestamp_offset =
            if let Some(audio_track_pipeline) = self.audio_track_pipelines.first() {
                let resample_ratio = self.configuration.output.sample_rate as f64
                    / audio_track_pipeline.configuration.track.sample_rate as f64;

                let output_channels = self.configuration.output.channels;

                (decoder_timestamp as f64 * resample_ratio).round() as u64 * output_channels as u64
            } else {
                0
            };

        self.samples_played_timestamp_offset
            .store(timestamp_offset, Ordering::Relaxed);

        self.generation_counter
            .audio_pipeline
            .fetch_add(1, Ordering::Release);
    }

    #[instrument(skip_all, level = "trace", fields(
        current_track = self
            .audio_track_pipelines
            .first()
            .map(|audio_track_pipeline| {
                Path::new(&audio_track_pipeline.configuration.track.file_path)
                    .file_name()
                    .unwrap_or_else(|| audio_track_pipeline.configuration.track.file_path.as_ref()).to_str()
            })
        ),
    )]
    pub fn process(&mut self) -> Result<AudioPipelineProcessDirective, AudioPipelineError> {
        let command = self.command_receiver.receive(&self.status)?;

        if let Some(command) = command {
            match self.handle_command(command) {
                Ok(Some(directive)) => return Ok(directive),
                Err(error) => return Err(error),
                _ => {}
            }
        }

        match self.audio_sink.write() {
            Err(AudioSinkError::FullRingBuffer) => {
                let output_format = &self.configuration.output;

                let sleep_duration_milliseconds =
                    ((SAMPLE_BUFFER_CAPACITY as f32 / output_format.channels as f32) * 1000.0
                        / output_format.sample_rate as f32)
                        * 0.5;

                return Ok(AudioPipelineProcessDirective::Sleep(Duration::from_millis(
                    sleep_duration_milliseconds.ceil() as u64,
                )));
            }
            Err(AudioSinkError::AwaitingBufferClear) => {
                return Ok(AudioPipelineProcessDirective::Sleep(Duration::from_millis(
                    1,
                )));
            }
            Ok(()) => {}
        }

        let Some(audio_track_pipeline) = self.audio_track_pipelines.get_mut(0) else {
            self.set_status(AudioPipelineStatus::Idle);

            return Ok(AudioPipelineProcessDirective::Continue);
        };

        if audio_track_pipeline.status == AudioTrackPipelineStatus::Finished {
            if self.audio_sink.is_empty() && self.audio_sink.is_engine_buffer_empty() {
                self.configuration
                    .event_emitter
                    .emit(AudioPipelineThreadEvent::TrackFinished);

                self.set_status(AudioPipelineStatus::Idle);

                return Ok(AudioPipelineProcessDirective::Continue);
            }

            return Ok(AudioPipelineProcessDirective::Sleep(Duration::from_millis(
                10,
            )));
        }

        let samples = audio_track_pipeline.produce_samples()?;

        self.audio_sink.buffer(samples.as_ref());

        Ok(AudioPipelineProcessDirective::Continue)
    }
}
