use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, SendError, Sender},
    thread::JoinHandle,
};

use rtrb::RingBuffer;
use thiserror::Error;
use tracing::error;

use crate::playback::{
    constants::SAMPLE_BUFFER_CAPACITY,
    engine::{PlaybackEngine, PlaybackEngineError},
    pipeline::{spawn_audio_pipeline_thread, AudioPipelineCommand, AudioPipelineEvent},
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
    audio_pipeline_event_receiver: Receiver<AudioPipelineEvent>,
    audio_pipeline_command_sender: Sender<AudioPipelineCommand>,
    audio_pipeline_thread_handle: Option<JoinHandle<()>>,

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
        let (
            audio_pipeline_thread_handle,
            audio_pipeline_command_sender,
            audio_pipeline_event_receiver,
        ) = spawn_audio_pipeline_thread();

        PlaybackController {
            audio_pipeline_thread_handle: Some(audio_pipeline_thread_handle),
            audio_pipeline_command_sender,
            audio_pipeline_event_receiver,
            playback_engine,
        }
    }

    pub fn initialize_output(&mut self) -> Result<(), PlaybackControllerError> {
        let (sample_buffer_producer, sample_buffer_consumer) =
            RingBuffer::new(SAMPLE_BUFFER_CAPACITY);

        let (sample_rate, channels) = self.playback_engine.build_stream(sample_buffer_consumer)?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineCommand::ChangeConfiguration {
                sample_rate: sample_rate,
                channels: channels,
                producer: sample_buffer_producer,
            })?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PlaybackControllerError> {
        self.playback_engine.pause_stream()?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineCommand::Pause)?;

        Ok(())
    }

    pub fn play(&mut self, track_path: Option<PathBuf>) -> Result<(), PlaybackControllerError> {
        self.playback_engine.play_stream()?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineCommand::Play(track_path))?;

        Ok(())
    }
}

impl Drop for PlaybackController {
    fn drop(&mut self) {
        match self
            .audio_pipeline_command_sender
            .send(AudioPipelineCommand::Exit)
        {
            Ok(_) => {
                if let Some(audio_pipeline_thread_handle) = self.audio_pipeline_thread_handle.take()
                {
                    if let Err(error) = audio_pipeline_thread_handle.join() {
                        error!("Audio pipeline thread join failed: {:#?}", error);
                    };
                }
            }
            Err(error) => {
                error!("Could not issue exit command to audio pipeline: {error}")
            }
        };
    }
}
