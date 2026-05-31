use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, RecvError, Sender, TryRecvError},
        Arc,
    },
    thread,
    time::Duration,
};

use rtrb::Producer;
use rubato::ResamplerConstructionError;
use symphonia::core::errors::Error as SymphoniaError;
use thiserror::Error;
use tracing::{error, info, info_span};

use crate::playback::{
    constants::SAMPLE_BUFFER_CAPACITY,
    pipeline::{
        channel_converter::AudioChannelConverter,
        decoder::{AudioDecoder, AudioDecoderError},
        resampler::{AudioResampler, AudioResamplerError},
        sink::AudioSink,
    },
};

pub mod channel_converter;
pub mod decoder;
pub mod resampler;
pub mod sink;

#[derive(Debug, Error, Clone)]
pub enum AudioPipelineError {
    #[error("missing audio track on selected file")]
    MissingAudioTrack,

    #[error("missing codec parameters on selected file")]
    MissingCodecParameters,

    #[error("missing resampler parameters")]
    MissingResamplerParameters,

    #[error("missing remixing parameters")]
    MissingRemixingParameters,

    #[error("unsupported remixing: channels go from {0} to {1}.")]
    UnsupportedRemixing(u16, u16),

    #[error("failed to build resampler: {0}")]
    ResamplerBuildFailed(Arc<ResamplerConstructionError>),

    #[error("io error: {0}")]
    Io(Arc<std::io::Error>),

    #[error("symphonia error: {0}")]
    Symphonia(Arc<SymphoniaError>),

    #[error("audio decoder error: {0}")]
    AudioDecoder(#[from] AudioDecoderError),

    #[error("audio resampler error: {0}")]
    AudioResampler(#[from] AudioResamplerError),
}

impl From<std::io::Error> for AudioPipelineError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(Arc::new(error))
    }
}

impl From<SymphoniaError> for AudioPipelineError {
    fn from(error: SymphoniaError) -> Self {
        Self::Symphonia(Arc::new(error))
    }
}

impl From<ResamplerConstructionError> for AudioPipelineError {
    fn from(error: ResamplerConstructionError) -> Self {
        Self::ResamplerBuildFailed(Arc::new(error))
    }
}

#[derive(Debug, Error)]
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

#[derive(Debug)]
pub struct AudioPipelineConfiguration {
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_buffer_producer: Producer<f32>,
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
    Exit,
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
}

pub fn spawn_audio_pipeline_thread() -> Sender<AudioPipelineCommand> {
    let (command_sender, command_receiver) = mpsc::channel();

    let mut audio_pipeline = AudioPipeline::new(command_receiver);

    std::thread::spawn(move || loop {
        let span = info_span!(parent: None, "audio_decoding_loop");
        let _guard = span.entered();

        // Get audio pipeline command blocking or non blocking depending on status.
        let pipeline_command_receive_result: Result<
            AudioPipelineCommand,
            AudioPipelineCommandReceiverError,
        > = if matches!(audio_pipeline.status, AudioPipelineStatus::Idle) {
            audio_pipeline
                .command_receiver
                .recv()
                .map_err(AudioPipelineCommandReceiverError::ReceiveFailed)
        } else {
            audio_pipeline
                .command_receiver
                .try_recv()
                .map_err(AudioPipelineCommandReceiverError::ReceiveAttemptFailed)
        };

        match pipeline_command_receive_result {
            Ok(pipeline_command) => {
                let pipeline_command_result = match pipeline_command {
                    AudioPipelineCommand::Exit => break,
                    AudioPipelineCommand::Play(track_path) => audio_pipeline.play(track_path),
                    AudioPipelineCommand::ChangeConfiguration {
                        sample_rate,
                        channels,
                        producer,
                    } => audio_pipeline.set_output_configuration(
                        AudioFormat {
                            sample_rate,
                            channels,
                        },
                        producer,
                    ),
                    AudioPipelineCommand::Stop => Ok(audio_pipeline.stop()),
                    AudioPipelineCommand::Pause => Ok(audio_pipeline.pause()),
                    AudioPipelineCommand::Seek(_) => todo!(),
                };

                match pipeline_command_result {
                    Ok(_) => {}
                    Err(error) => {
                        error!("{}", error.to_string())
                    }
                }
            }

            Err(error) => match error {
                AudioPipelineCommandReceiverError::ReceiveFailed(_) => {
                    error!("{}", error.to_string())
                }
                AudioPipelineCommandReceiverError::ReceiveAttemptFailed(_) => {}
            },
        };

        let Some(decoder) = audio_pipeline.decoder.as_mut() else {
            continue;
        };

        let Some(output_format) = audio_pipeline.output_format.as_ref() else {
            continue;
        };

        let Some(audio_sink) = audio_pipeline.sink.as_mut() else {
            continue;
        };

        match audio_sink.write() {
            Ok(_) => {}
            Err(_) => {
                // Sleep 50% of the time it takes to drain the buffer.
                let sleep_duration_milliseconds =
                    ((SAMPLE_BUFFER_CAPACITY as f32 / output_format.channels as f32) * 1000.0
                        / output_format.sample_rate as f32)
                        * 0.5;

                thread::sleep(Duration::from_millis(
                    sleep_duration_milliseconds.ceil() as u64
                ));
            }
        }

        if !audio_sink.is_empty() {
            continue;
        }
        // Resampling & Remixing
        let mut decoded_samples: Vec<f32>;

        match decoder.decode() {
            Ok(Some(samples)) => decoded_samples = samples,
            Ok(None) => {
                info!("No more packets from demuxer.");
                audio_pipeline.status = AudioPipelineStatus::Idle;

                continue;
            }
            Err(AudioDecoderError::RecoverableDecoderError(error)) => {
                error!("Decoder error: {error}");

                continue;
            }
            Err(error) => {
                error!("Decoder error: {error}");

                audio_pipeline.status = AudioPipelineStatus::Idle;

                continue;
            }
        };

        if decoder.track.channels > output_format.channels {
            match AudioChannelConverter::convert(
                &decoded_samples,
                decoder.track.channels,
                output_format.channels,
            ) {
                Ok(remixed_samples) => decoded_samples = remixed_samples,
                Err(error) => {
                    error!("Could not remix samples: {error}");

                    audio_pipeline.status = AudioPipelineStatus::Idle;

                    continue;
                }
            }
        }

        if let Some(resampler) = audio_pipeline.resampler.as_mut() {
            let mut resampled_samples: Vec<f32>;

            match resampler.resample(&decoded_samples) {
                Ok(samples) => resampled_samples = samples,
                Err(error) => {
                    error!("Could not resample samples: {error}");

                    audio_pipeline.status = AudioPipelineStatus::Idle;

                    continue;
                }
            }

            if decoder.track.channels < output_format.channels {
                match AudioChannelConverter::convert(
                    &resampled_samples,
                    decoder.track.channels,
                    output_format.channels,
                ) {
                    Ok(remixed_samples) => resampled_samples = remixed_samples,
                    Err(error) => {
                        error!("Could not remix samples: {error}");

                        audio_pipeline.status = AudioPipelineStatus::Idle;

                        continue;
                    }
                }
            }

            audio_sink.buffer(&resampled_samples);
        } else {
            if decoder.track.channels < output_format.channels {
                match AudioChannelConverter::convert(
                    &decoded_samples,
                    decoder.track.channels,
                    output_format.channels,
                ) {
                    Ok(remixed_samples) => decoded_samples = remixed_samples,
                    Err(error) => {
                        error!("Could not remix samples: {error}");

                        audio_pipeline.status = AudioPipelineStatus::Idle;

                        continue;
                    }
                }
            }

            audio_sink.buffer(&decoded_samples);
        }
    });

    command_sender
}
