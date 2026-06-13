use std::ops::ControlFlow;

use thiserror::Error;
use tracing::{error, instrument, warn};

use crate::{
    playback::pipeline::{
        command::{AudioPipelineCommandReceiver, AudioPipelineCommandReceiverError},
        config::AudioPipelineConfiguration,
        sink::{AudioSink, AudioSinkError},
        stage::{
            channel_converter::AudioChannelConverterError, decoder::AudioDecoderError,
            resampler::AudioResamplerError,
        },
        thread::{AudioPipelineThreadCommand, AudioPipelineThreadEvent},
        track::{AudioTrackPipeline, AudioTrackPipelineStatus},
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
    #[error("There are no more samples to decode for current track.")]
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
}

pub enum AudioPipelineStatus {
    Idle,
    ProducingSamples,
    Paused,
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
    ) -> Self {
        let mut audio_pipeline = Self {
            audio_sink,
            command_receiver,
            configuration,
            audio_track_pipelines: Vec::new(),
            status: AudioPipelineStatus::Idle,
        };

        if let Some(track) = track {
            if let Err(error) = audio_pipeline.play_track(track) {
                error!(
                    "Failed to create audio track pipeline when creating audio pipeline {error}"
                );
            }
        }

        audio_pipeline
    }

    #[instrument(skip_all)]
    pub fn play_track(&mut self, track: Track) -> Result<(), AudioPipelineError> {
        let audio_track_pipeline = AudioTrackPipeline::build(track, self.configuration.clone())?;

        self.audio_track_pipelines = vec![audio_track_pipeline];

        self.status = AudioPipelineStatus::ProducingSamples;

        Ok(())
    }

    pub fn pause(&mut self) {
        self.status = AudioPipelineStatus::Idle;
    }

    pub fn stop(&mut self) {
        self.status = AudioPipelineStatus::Idle;

        self.audio_sink.clear();
    }

    #[instrument(skip_all)]
    pub fn resume(&mut self) {
        let Some(audio_track_pipeline) = self.audio_track_pipelines.get_mut(0) else {
            warn!("Attempted to resume playback without a track to play.");

            return;
        };

        if matches!(
            audio_track_pipeline.status,
            AudioTrackPipelineStatus::Ongoing | AudioTrackPipelineStatus::Ready
        ) {
            self.status = AudioPipelineStatus::ProducingSamples;
        }
        // TODO: Add track replay if the status of the audio track pipeline is `Finished`.
    }

    #[instrument(skip(self))]
    pub fn handle_command(
        &mut self,
        command: AudioPipelineThreadCommand,
    ) -> Result<ControlFlow<(), ()>, AudioPipelineError> {
        if let Some(audio_track_pipeline) = self.audio_track_pipelines.get_mut(0) {
            audio_track_pipeline.handle_command(&command)?;
        }

        match command {
            AudioPipelineThreadCommand::Play(track) => self.play_track(track)?,
            AudioPipelineThreadCommand::Pause => self.pause(),
            AudioPipelineThreadCommand::Resume => self.resume(),
            AudioPipelineThreadCommand::Stop => self.stop(),
            AudioPipelineThreadCommand::PlayNext => todo!(),
            AudioPipelineThreadCommand::PlayPrevious => todo!(),
            AudioPipelineThreadCommand::Seek(_) => todo!(),
            AudioPipelineThreadCommand::ChangeNextTrack(_) => todo!(),
            AudioPipelineThreadCommand::ChangeOutput {
                output: _,
                audio_engine_producer: _,
            } => {
                todo!()
            }
            AudioPipelineThreadCommand::Exit => return Ok(ControlFlow::Break(())),
        };

        Ok(ControlFlow::Continue(()))
    }

    #[instrument(skip_all, err)]
    pub fn process(&mut self) -> Result<ControlFlow<(), ()>, AudioPipelineError> {
        let command = self.command_receiver.receive(&self.status)?;

        if let Some(command) = command {
            match self.handle_command(command) {
                Ok(ControlFlow::Continue(_)) => {}
                Ok(ControlFlow::Break(_)) => return Ok(ControlFlow::Break(())),
                Err(_) => {}
            }
        }

        self.audio_sink.write()?;

        let Some(audio_track_pipeline) = self.audio_track_pipelines.get_mut(0) else {
            self.status = AudioPipelineStatus::Idle;

            return Ok(ControlFlow::Continue(()));
        };

        if self.audio_sink.is_empty()
            && matches!(
                audio_track_pipeline.status,
                AudioTrackPipelineStatus::Finished
            )
        {
            self.configuration
                .event_emitter
                .emit(AudioPipelineThreadEvent::TrackFinished);

            self.status = AudioPipelineStatus::Idle;

            return Ok(ControlFlow::Continue(()));
        }

        let samples = audio_track_pipeline.produce_samples()?;

        self.audio_sink.buffer(&samples.as_ref());

        Ok(ControlFlow::Continue(()))
    }
}
