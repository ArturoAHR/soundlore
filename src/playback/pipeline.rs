use std::{
    collections::VecDeque,
    fs::File,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, RecvError, Sender, TryRecvError},
        Arc,
    },
    thread,
    time::Duration,
};

use rtrb::Producer;
use symphonia::{
    core::{
        codecs::{
            audio::{AudioDecoder, AudioDecoderOptions},
            CodecParameters,
        },
        errors::Error as SymphoniaError,
        formats::{probe::Hint, FormatOptions, FormatReader, TrackType},
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default::get_codecs,
};
use thiserror::Error;
use tracing::error;

use crate::playback::constants::SAMPLE_BUFFER_CAPACITY;

#[derive(Debug, Error, Clone)]
pub enum AudioPipelineError {
    #[error("missing audio track on selected file")]
    MissingAudioTrack,

    #[error("missing codec parameters on selected file")]
    MissingCodecParameters,

    #[error("io error: {0}")]
    Io(Arc<std::io::Error>),

    #[error("symphonia error: {0}")]
    Symphonia(Arc<SymphoniaError>),
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

#[derive(Debug, Error)]
pub enum AudioPipelineCommandReceiverError {
    #[error("Failed to receive audio pipeline command: {0}")]
    ReceiveFailed(#[from] RecvError),

    #[error("There are no pending audio pipeline commands in the channel: {0}")]
    ReceiveAttemptFailed(#[from] TryRecvError),
}

pub struct AudioPipeline {
    pub command_receiver: Receiver<AudioPipelineCommand>,
    pub configuration: Option<AudioPipelineConfiguration>,
    pub current_track: Option<AudioPipelineTrack>,
    pub status: AudioPipelineStatus,
}

pub enum AudioPipelineStatus {
    ProducingSamples,
    Idle,
}

pub struct AudioPipelineTrack {
    path: PathBuf,
    audio_track_id: u32,
    demuxer: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    sample_buffer: VecDeque<f32>,
}

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
            configuration: None,
            current_track: None,
            status: AudioPipelineStatus::Idle,
        }
    }

    pub fn set_current_track(&mut self, track_path: PathBuf) -> Result<(), AudioPipelineError> {
        let track_file = File::open(&track_path)?;

        let media_source_stream = MediaSourceStream::new(Box::new(track_file), Default::default());

        let mut hint = Hint::new();

        if let Some(file_format) = track_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
        {
            hint.with_extension(&file_format);
        }

        let demuxer = symphonia::default::get_probe().probe(
            &hint,
            media_source_stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        let track = demuxer
            .first_track_known_codec(TrackType::Audio)
            .ok_or_else(|| AudioPipelineError::MissingAudioTrack)?;

        let Some(CodecParameters::Audio(codec_params)) = track.codec_params.as_ref() else {
            return Err(AudioPipelineError::MissingCodecParameters.into());
        };

        let decoder =
            get_codecs().make_audio_decoder(&codec_params, &AudioDecoderOptions::default())?;

        self.current_track = Some(AudioPipelineTrack {
            path: track_path,
            audio_track_id: track.id,
            demuxer,
            decoder,
            sample_buffer: VecDeque::new(),
        });

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

        self.set_current_track(track_path)?;

        self.status = AudioPipelineStatus::ProducingSamples;

        Ok(())
    }

    pub fn update_pipeline_configuration(&mut self, configuration: AudioPipelineConfiguration) {
        self.configuration = Some(configuration);
    }
}

pub fn spawn_audio_pipeline_thread() -> Sender<AudioPipelineCommand> {
    let (command_sender, command_receiver) = mpsc::channel();

    let mut audio_pipeline = AudioPipeline::new(command_receiver);

    std::thread::spawn(move || loop {
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
                    } => Ok(audio_pipeline.update_pipeline_configuration(
                        AudioPipelineConfiguration {
                            sample_rate,
                            channels,
                            sample_buffer_producer: producer,
                        },
                    )),
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

        if let Some(track) = audio_pipeline.current_track.as_mut() {
            while !track.sample_buffer.is_empty() {
                if let Some(configuration) = audio_pipeline.configuration.as_mut() {
                    let Some(sample) = track.sample_buffer.front() else {
                        break;
                    };

                    match configuration.sample_buffer_producer.push(*sample) {
                        Ok(_) => {
                            track.sample_buffer.pop_front();
                        }
                        Err(_) => {
                            // Sleep 50% of the time it takes to drain the buffer.
                            let sleep_duration_milliseconds = ((SAMPLE_BUFFER_CAPACITY as f32
                                / configuration.channels as f32)
                                * 1000.0
                                / configuration.sample_rate as f32)
                                * 0.5;

                            thread::sleep(Duration::from_millis(
                                sleep_duration_milliseconds.ceil() as u64,
                            ));

                            break;
                        }
                    };
                }
            }

            if !track.sample_buffer.is_empty() {
                continue;
            }

            let mut packet = None;

            while let Ok(Some(current_packet)) = track.demuxer.next_packet() {
                if current_packet.track_id == track.audio_track_id {
                    packet = Some(current_packet);
                    break;
                }
            }

            let Some(packet) = packet else {
                audio_pipeline.status = AudioPipelineStatus::Idle;

                continue;
            };

            let generic_audio_buffer = match track.decoder.decode(&packet) {
                Ok(generic_audio_buffer) => generic_audio_buffer,
                Err(decode_error) => {
                    error!("Failed to decode audio: {}", decode_error);

                    match decode_error {
                        SymphoniaError::ResetRequired => {
                            track.decoder.reset();
                        }
                        SymphoniaError::DecodeError(_) => {}
                        SymphoniaError::IoError(_) => {}
                        _ => {
                            error!("Stopping decoding.");

                            audio_pipeline.status = AudioPipelineStatus::Idle;
                        }
                    }

                    continue;
                }
            };

            let mut samples: Vec<f32> = Vec::new();

            generic_audio_buffer.copy_to_vec_interleaved(&mut samples);

            track.sample_buffer.extend(samples);
        }
    });

    command_sender
}
