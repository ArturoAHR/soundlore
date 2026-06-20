use std::{
    cmp::max,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
        mpsc::{Receiver, SendError, Sender, TryRecvError},
    },
    thread::JoinHandle,
};

use rtrb::RingBuffer;
use thiserror::Error;
use tracing::error;

use crate::{
    playback::{
        constants::SAMPLE_BUFFER_CAPACITY,
        engine::{PlaybackEngine, PlaybackEngineError},
        pipeline::{
            AudioFormat,
            thread::{
                AudioPipelineThreadCommand, AudioPipelineThreadEvent, spawn_audio_pipeline_thread,
            },
        },
    },
    track::models::Track,
};

pub mod constants;
pub mod engine;
pub mod pipeline;

#[derive(Debug, Error, Clone)]
pub enum PlaybackControllerError {
    #[error("failed to send command to pipeline: {0}")]
    AudioPipelineCommandSendFailed(String),

    #[error("failed to poll event from the pipeline: {0}")]
    AudioPipelineEventReceiveAttemptFailed(String),

    #[error("playback error - {0}")]
    PlaybackEngine(#[from] PlaybackEngineError),
}

pub struct PlaybackController {
    audio_pipeline_event_receiver: Receiver<AudioPipelineThreadEvent>,
    audio_pipeline_command_sender: Sender<AudioPipelineThreadCommand>,
    audio_pipeline_thread_handle: Option<JoinHandle<()>>,

    playback_engine: Box<dyn PlaybackEngine>,

    samples_played: Arc<AtomicU64>,
    track_start_timestamp: Arc<AtomicI64>,
    samples_played_timestamp_offset: Arc<AtomicU64>,
    generation_counter: Arc<GenerationCounter>,
}

pub struct GenerationCounter {
    pub audio_engine: AtomicU64,
    pub audio_pipeline: AtomicU64,
}

#[derive(Debug, Clone)]
pub enum Event {
    Initialized,
    UnexpectedError(PlaybackControllerError),
    PendingOutputDeviceChange,
    OutputDeviceChanged,
    AudioPipelineEventPollTick,
}

pub enum PlaybackControllerCommand {
    Play(Track),
    Stop,
    Pause,
    Resume,
    Seek(u64),
}

impl From<SendError<AudioPipelineThreadCommand>> for PlaybackControllerError {
    fn from(error: SendError<AudioPipelineThreadCommand>) -> Self {
        Self::AudioPipelineCommandSendFailed(error.to_string())
    }
}

impl PlaybackController {
    pub fn new(playback_engine: Box<dyn PlaybackEngine>) -> Self {
        let samples_played = Arc::new(AtomicU64::new(0));
        let generation_counter = Arc::new(GenerationCounter {
            audio_engine: AtomicU64::new(0),
            audio_pipeline: AtomicU64::new(0),
        });
        let track_start_timestamp = Arc::new(AtomicI64::new(0));
        let samples_played_timestamp_offset = Arc::new(AtomicU64::new(0));

        let (
            audio_pipeline_thread_handle,
            audio_pipeline_command_sender,
            audio_pipeline_event_receiver,
        ) = spawn_audio_pipeline_thread(
            Arc::clone(&samples_played_timestamp_offset),
            Arc::clone(&generation_counter),
        );

        PlaybackController {
            audio_pipeline_thread_handle: Some(audio_pipeline_thread_handle),
            audio_pipeline_command_sender,
            audio_pipeline_event_receiver,
            playback_engine,

            samples_played: Arc::clone(&samples_played),
            generation_counter: Arc::clone(&generation_counter),
            track_start_timestamp: Arc::clone(&track_start_timestamp),
            samples_played_timestamp_offset: Arc::clone(&samples_played_timestamp_offset),
        }
    }

    pub fn initialize_output(&mut self) -> Result<(), PlaybackControllerError> {
        let (sample_buffer_producer, sample_buffer_consumer) =
            RingBuffer::new(SAMPLE_BUFFER_CAPACITY);

        let (sample_rate, channels) = self.playback_engine.build_stream(
            sample_buffer_consumer,
            Arc::clone(&self.samples_played),
            Arc::clone(&self.track_start_timestamp),
            Arc::clone(&self.samples_played_timestamp_offset),
            Arc::clone(&self.generation_counter),
        )?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineThreadCommand::ChangeOutput {
                output: AudioFormat {
                    sample_rate,
                    channels,
                },
                audio_engine_producer: sample_buffer_producer,
            })?;

        Ok(())
    }

    pub fn play(&mut self, track: Track) -> Result<(), PlaybackControllerError> {
        self.playback_engine.play_stream()?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineThreadCommand::Play(track))?;

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), PlaybackControllerError> {
        self.playback_engine.play_stream()?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineThreadCommand::Resume)?;

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), PlaybackControllerError> {
        self.playback_engine.pause_stream()?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineThreadCommand::Pause)?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PlaybackControllerError> {
        self.playback_engine.pause_stream()?;

        self.audio_pipeline_command_sender
            .send(AudioPipelineThreadCommand::Stop)?;

        Ok(())
    }

    pub fn seek(&mut self, timestamp: u64) -> Result<(), PlaybackControllerError> {
        self.audio_pipeline_command_sender
            .send(AudioPipelineThreadCommand::Seek(timestamp))?;

        Ok(())
    }

    pub fn poll_audio_pipeline_event(
        &self,
    ) -> Result<Option<AudioPipelineThreadEvent>, PlaybackControllerError> {
        match self.audio_pipeline_event_receiver.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(error) => Err(
                PlaybackControllerError::AudioPipelineEventReceiveAttemptFailed(error.to_string()),
            ),
        }
    }

    pub fn current_track_samples_played(&self) -> u64 {
        let track_start_timestamp = self.track_start_timestamp.load(Ordering::Acquire);
        let samples_played = self.samples_played.load(Ordering::Relaxed);

        return max(0, samples_played as i64 - track_start_timestamp) as u64;
    }
}

impl Drop for PlaybackController {
    fn drop(&mut self) {
        match self
            .audio_pipeline_command_sender
            .send(AudioPipelineThreadCommand::Exit)
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
