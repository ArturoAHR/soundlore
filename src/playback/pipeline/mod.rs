use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, RecvError, Sender, TryRecvError},
    thread,
    time::Duration,
};

use rtrb::Producer;
use thiserror::Error;
use tracing::{error, info, info_span};

use crate::playback::{
    constants::SAMPLE_BUFFER_CAPACITY,
    pipeline::{
        channel_converter::{AudioChannelConverter, AudioChannelConverterError},
        decoder::{AudioDecoder, AudioDecoderError},
        resampler::{AudioResampler, AudioResamplerError},
        sink::{AudioSink, AudioSinkError},
    },
};

pub mod channel_converter;
pub mod decoder;
pub mod resampler;
pub mod sink;

#[derive(Debug, Error, Clone)]
pub enum AudioPipelineError {
    #[error("missing resampler parameters")]
    MissingResamplerParameters,

    #[error("audio pipeline command receiver error: {0}")]
    AudioPipelineCommandReceiver(#[from] AudioPipelineCommandReceiverError),

    #[error("audio decoder error: {0}")]
    AudioDecoder(#[from] AudioDecoderError),

    #[error("audio resampler error: {0}")]
    AudioResampler(#[from] AudioResamplerError),

    #[error("audio channel converter error: {0}")]
    AudioChannelConverter(#[from] AudioChannelConverterError),

    #[error("audio sink error: {0}")]
    AudioSink(#[from] AudioSinkError),
}

#[derive(Debug, Error, Clone)]
pub enum AudioPipelineCommandReceiverError {
    #[error("Failed to receive audio pipeline command: {0}")]
    ReceiveFailed(#[from] RecvError),

    #[error("There are no pending audio pipeline commands in the channel: {0}")]
    ReceiveAttemptFailed(#[from] TryRecvError),
}

pub struct AudioPipeline {
    pub command_receiver: Receiver<AudioPipelineCommand>,
    pub decoder: Option<AudioDecoder>,
    pub sink: Option<AudioSink>,
    pub resampler: Option<AudioResampler>,
    pub status: AudioPipelineStatus,
    pub output_format: Option<AudioFormat>,
}

#[derive(Debug)]
pub struct AudioFormat {
    sample_rate: u32,
    channels: u16,
}

pub enum AudioPipelineStatus {
    ProducingSamples,
    Idle,
}

pub enum AudioPipelineCommand {
    Play(Option<PathBuf>),
    Pause,
    Stop,
    Seek(f32),
    ChangeConfiguration {
        sample_rate: u32,
        channels: u16,
        producer: Producer<f32>,
    },
}

impl AudioPipeline {
    pub fn new(command_receiver: Receiver<AudioPipelineCommand>) -> Self {
        Self {
            command_receiver,
            status: AudioPipelineStatus::Idle,
            resampler: None,
            decoder: None,
            output_format: None,
            sink: None,
        }
    }

    pub fn pause(&mut self) {
        self.status = AudioPipelineStatus::Idle;
    }

    pub fn stop(&mut self) {
        // TODO: Reset the decoding position to start of the file.
        self.status = AudioPipelineStatus::Idle;
    }

    pub fn play(&mut self, track_path: Option<PathBuf>) -> Result<(), AudioPipelineError> {
        let Some(track_path) = track_path else {
            return Ok(());
        };

        self.build_decoder(track_path)?;

        self.status = AudioPipelineStatus::ProducingSamples;

        Ok(())
    }

    /// Get audio pipeline command blocking or non blocking depending on status.
    pub fn receive_command(&self) -> Result<Option<AudioPipelineCommand>, AudioPipelineError> {
        if matches!(self.status, AudioPipelineStatus::Idle) {
            let command = self
                .command_receiver
                .recv()
                .map_err(AudioPipelineCommandReceiverError::ReceiveFailed)?;

            return Ok(Some(command));
        }

        Ok(self.command_receiver.try_recv().ok())
    }

    pub fn handle_command(
        &mut self,
        command: AudioPipelineCommand,
    ) -> Result<(), AudioPipelineError> {
        match command {
            AudioPipelineCommand::Play(track_path) => {
                self.play(track_path)?;
            }
            AudioPipelineCommand::ChangeConfiguration {
                sample_rate,
                channels,
                producer,
            } => {
                self.set_output_configuration(
                    AudioFormat {
                        sample_rate,
                        channels,
                    },
                    producer,
                )?;
            }
            AudioPipelineCommand::Stop => {
                self.stop();
            }
            AudioPipelineCommand::Pause => {
                self.pause();
            }
            AudioPipelineCommand::Seek(_) => todo!(),
        };

        Ok(())
    }

    pub fn build_decoder(&mut self, track_path: PathBuf) -> Result<(), AudioPipelineError> {
        let decoder = AudioDecoder::build(track_path)?;

        self.decoder = Some(decoder);

        if self.output_format.is_some() {
            self.build_resampler()?;
        }

        Ok(())
    }

    pub fn build_resampler(&mut self) -> Result<(), AudioPipelineError> {
        let Some(decoder) = self.decoder.as_ref() else {
            return Err(AudioPipelineError::MissingResamplerParameters);
        };

        let Some(output_format) = self.output_format.as_ref() else {
            return Err(AudioPipelineError::MissingResamplerParameters);
        };

        if decoder.track.sample_rate == output_format.sample_rate {
            self.resampler = None;

            return Ok(());
        }

        let resampler = AudioResampler::build(
            decoder.track.sample_rate,
            decoder.track.channels,
            output_format.sample_rate,
            output_format.channels,
        )?;

        self.resampler = Some(resampler);

        Ok(())
    }

    pub fn set_output_configuration(
        &mut self,
        output_format: AudioFormat,
        audio_engine_producer: Producer<f32>,
    ) -> Result<(), AudioPipelineError> {
        info!(
            "Configuration set, output sample rate is {}Hz with {} channels",
            output_format.sample_rate, output_format.channels
        );

        let audio_sink = AudioSink::new(audio_engine_producer);

        self.output_format = Some(output_format);
        self.sink = Some(audio_sink);

        if self.decoder.is_some() {
            self.build_resampler()?;
        }

        Ok(())
    }

    /// Decoder thread processing
    pub fn process(&mut self) -> Result<(), AudioPipelineError> {
        let mut audio_pipeline_command = None;
        match self.receive_command() {
            Ok(command) => audio_pipeline_command = command,
            Err(error) => {
                error!("Audio pipeline command receive failed: {error}")
            }
        }

        if let Some(audio_pipeline_command) = audio_pipeline_command {
            match self.handle_command(audio_pipeline_command) {
                Ok(_) => {}
                Err(error) => {
                    error!("Audio pipeline command processing failed: {error}")
                }
            }
        }

        let Some(decoder) = self.decoder.as_mut() else {
            return Ok(());
        };

        let Some(output_format) = self.output_format.as_ref() else {
            return Ok(());
        };

        let Some(audio_sink) = self.sink.as_mut() else {
            return Ok(());
        };

        audio_sink.write()?;

        let Some(mut decoded_samples) = decoder.decode()? else {
            info!("No more packets from demuxer.");

            self.status = AudioPipelineStatus::Idle;

            return Ok(());
        };

        if decoder.track.channels > output_format.channels {
            decoded_samples = AudioChannelConverter::convert(
                &decoded_samples,
                decoder.track.channels,
                output_format.channels,
            )?;
        }

        if let Some(resampler) = self.resampler.as_mut() {
            let mut resampled_samples = resampler.resample(&decoded_samples)?;

            if decoder.track.channels < output_format.channels {
                resampled_samples = AudioChannelConverter::convert(
                    &resampled_samples,
                    decoder.track.channels,
                    output_format.channels,
                )?;
            }

            audio_sink.buffer(&resampled_samples);
        } else {
            if decoder.track.channels < output_format.channels {
                decoded_samples = AudioChannelConverter::convert(
                    &decoded_samples,
                    decoder.track.channels,
                    output_format.channels,
                )?;
            }

            audio_sink.buffer(&decoded_samples);
        }

        Ok(())
    }
}

pub fn spawn_audio_pipeline_thread() -> Sender<AudioPipelineCommand> {
    let (command_sender, command_receiver) = mpsc::channel();

    let mut audio_pipeline = AudioPipeline::new(command_receiver);

    std::thread::spawn(move || loop {
        let span = info_span!(parent: None, "audio_decoding_loop");
        let _guard = span.entered();

        match audio_pipeline.process() {
            Ok(_) => {}
            Err(AudioPipelineError::AudioSink(AudioSinkError::FullRingBuffer)) => {
                let Some(output_format) = audio_pipeline.output_format.as_ref() else {
                    audio_pipeline.status = AudioPipelineStatus::Idle;

                    continue;
                };

                let sleep_duration_milliseconds =
                    ((SAMPLE_BUFFER_CAPACITY as f32 / output_format.channels as f32) * 1000.0
                        / output_format.sample_rate as f32)
                        * 0.5;

                thread::sleep(Duration::from_millis(
                    sleep_duration_milliseconds.ceil() as u64
                ));
            }
            Err(AudioPipelineError::AudioDecoder(AudioDecoderError::RecoverableDecoderError(
                error,
            ))) => {
                error!("audio pipeline error: {error}");
            }
            Err(error) => {
                audio_pipeline.status = AudioPipelineStatus::Idle;

                error!("audio pipeline error: {error}");
            }
        }
    });

    command_sender
}
