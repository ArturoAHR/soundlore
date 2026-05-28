use std::sync::mpsc::{SendError, Sender};

use rtrb::RingBuffer;
use thiserror::Error;

use crate::{
    error::AppError,
    playback::{
        constants::SAMPLE_BUFFER_CAPACITY,
        engine::PlaybackEngine,
        pipeline::{spawn_audio_pipeline_thread, AudioPipelineCommand},
    },
};

pub mod constants;
pub mod engine;
pub mod pipeline;

#[derive(Debug, Error, Clone)]
pub enum PlaybackControllerError {
    #[error("failed to send command to pipeline: {0}")]
    PipelineCommandSendFailed(String),
}

pub struct PlaybackController {
    pipeline_command_sender: Sender<AudioPipelineCommand>,
    playback_engine: PlaybackEngine,
}

impl From<SendError<AudioPipelineCommand>> for AppError {
    fn from(error: SendError<AudioPipelineCommand>) -> Self {
        Self::PlaybackController(PlaybackControllerError::PipelineCommandSendFailed(
            error.to_string(),
        ))
    }
}

impl PlaybackController {
    pub fn new() -> Self {
        let playback_engine = PlaybackEngine::new();

        let pipeline_command_sender = spawn_audio_pipeline_thread();

        PlaybackController {
            pipeline_command_sender,
            playback_engine,
        }
    }

    pub fn initialize_output(&mut self) -> Result<(), AppError> {
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

    pub fn stop(&mut self) -> Result<(), AppError> {
        self.playback_engine.pause_stream()?;

        self.pipeline_command_sender
            .send(AudioPipelineCommand::Pause)?;

        Ok(())
    }

    pub fn play(&mut self) -> Result<(), AppError> {
        self.playback_engine.play_stream()?;

        self.pipeline_command_sender
            .send(AudioPipelineCommand::Play(None))?;

        Ok(())
    }
}
