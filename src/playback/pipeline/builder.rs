use std::sync::{Arc, atomic::AtomicU64};

use rtrb::Producer;
use thiserror::Error;
use tracing::warn;

use crate::{
    playback::{
        GenerationCounter,
        pipeline::{
            AudioFormat, AudioPipeline,
            command::{AudioPipelineCommandReceiver, CommandReceiver},
            config::AudioPipelineConfiguration,
            event::{AudioPipelineEventEmitter, EventSender},
            sink::AudioSink,
            thread::AudioPipelineThreadCommand,
        },
    },
    track::models::Track,
};

#[derive(Debug, Error)]
pub enum AudioPipelineBuilderError {
    #[error("Missing parameter: {0}")]
    MissingParameters(String),

    #[error("Failed to receive audio pipeline command: {0}")]
    ReceiveFailed(#[from] std::sync::mpsc::RecvError),
}

/// Builder that gathers the necessary elements for the initial state of the audio pipeline.
pub struct AudioPipelineBuilder {
    track: Option<Track>,
    output: Option<AudioPipelineOutput>,
    event_emitter: AudioPipelineEventEmitter,
    command_receiver: CommandReceiver,
    // TODO: Add initial playback configuration here (example: transition type or volume)
    // configuration: AudioPipelineConfiguration,
    samples_played_timestamp_offset: Arc<AtomicU64>,
    generation_counter: Arc<GenerationCounter>,
}

pub struct AudioPipelineOutput {
    format: AudioFormat,
    audio_engine_producer: Producer<f32>,
}

impl AudioPipelineBuilder {
    pub fn new(
        command_receiver: CommandReceiver,
        event_sender: EventSender,
        samples_played_timestamp_offset: Arc<AtomicU64>,
        generation_counter: Arc<GenerationCounter>,
    ) -> Self {
        Self {
            track: None,
            output: None,
            event_emitter: AudioPipelineEventEmitter::new(event_sender),
            command_receiver,
            samples_played_timestamp_offset,
            generation_counter,
        }
    }

    pub fn receive_and_handle_command(&mut self) -> Result<bool, AudioPipelineBuilderError> {
        let command = self.command_receiver.recv()?;

        match command {
            AudioPipelineThreadCommand::Play(track) => {
                self.track = Some(track);
            }
            AudioPipelineThreadCommand::ChangeOutput {
                output,
                audio_engine_producer,
            } => {
                self.output = Some(AudioPipelineOutput {
                    format: output,
                    audio_engine_producer,
                });

                return Ok(true);
            }
            _ => {
                warn!(
                    "Unsupported command has been issued to builder: {:?}",
                    command
                );
            }
        }

        Ok(false)
    }

    /// Performs the necessary steps to produce an audio pipeline
    pub fn build(mut self) -> Result<AudioPipeline, AudioPipelineBuilderError> {
        loop {
            if self.receive_and_handle_command()? {
                break;
            }
        }

        let Some(output) = self.output else {
            return Err(AudioPipelineBuilderError::MissingParameters(
                "Output Configuration".to_owned(),
            ));
        };

        let configuration = AudioPipelineConfiguration {
            event_emitter: self.event_emitter,
            volume_percentage: 100,
            output: output.format,
        };

        let audio_sink = AudioSink::new(
            output.audio_engine_producer,
            Arc::clone(&self.generation_counter),
        );

        let command_receiver = AudioPipelineCommandReceiver::new(self.command_receiver);

        Ok(AudioPipeline::new(
            configuration,
            audio_sink,
            command_receiver,
            self.track,
            self.samples_played_timestamp_offset,
            self.generation_counter,
        ))
    }
}
