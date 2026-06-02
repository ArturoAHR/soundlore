use std::{
    path::PathBuf,
    sync::mpsc::{SendError, Sender},
};

use rtrb::RingBuffer;
use thiserror::Error;

use crate::playback::{
    constants::SAMPLE_BUFFER_CAPACITY,
    engine::{PlaybackEngine, PlaybackEngineError},
    pipeline::{spawn_audio_pipeline_thread, AudioPipelineCommand},
};

pub mod constants;
pub mod engine;
pub mod pipeline;

#[derive(Debug, Error, Clone)]
pub enum PlaybackControllerError {
    #[error("failed to send command to pipeline: {0}")]
    PipelineCommandSendFailed(String),

    #[error("playback error - {0}")]
    PlaybackEngine(#[from] PlaybackEngineError),
}

pub struct PlaybackController {
    pipeline_command_sender: Sender<AudioPipelineCommand>,
    playback_engine: Box<dyn PlaybackEngine>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Initialized,
    UnexpectedError(PlaybackControllerError),
}

pub enum PlaybackControllerCommand {
    Play(Option<PathBuf>),
    Stop,
    Pause,
    // TODO: Add Seek
}

impl From<SendError<AudioPipelineCommand>> for PlaybackControllerError {
    fn from(error: SendError<AudioPipelineCommand>) -> Self {
        Self::PipelineCommandSendFailed(error.to_string())
    }
}

impl PlaybackController {
    pub fn new(playback_engine: Box<dyn PlaybackEngine>) -> Self {
        let pipeline_command_sender = spawn_audio_pipeline_thread();

        PlaybackController {
            pipeline_command_sender,
            playback_engine,
        }
    }

    pub fn initialize_output(&mut self) -> Result<(), PlaybackControllerError> {
        let (sample_buffer_producer, sample_buffer_consumer) =
            RingBuffer::new(SAMPLE_BUFFER_CAPACITY);

        let (sample_rate, channels) = self.playback_engine.build_stream(sample_buffer_consumer)?;

        self.pipeline_command_sender
            .send(AudioPipelineCommand::ChangeConfiguration {
                sample_rate: sample_rate,
                channels: channels,
                producer: sample_buffer_producer,
            })?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PlaybackControllerError> {
        self.playback_engine.pause_stream()?;

        self.pipeline_command_sender
            .send(AudioPipelineCommand::Pause)?;

        Ok(())
    }

    pub fn play(&mut self, track_path: Option<PathBuf>) -> Result<(), PlaybackControllerError> {
        self.playback_engine.play_stream()?;

        self.pipeline_command_sender
            .send(AudioPipelineCommand::Play(track_path))?;

        Ok(())
    }
}
