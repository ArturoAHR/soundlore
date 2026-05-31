use std::{
    cmp::min,
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
use rubato::{
    audioadapter_buffers::direct::InterleavedSlice, Fft, FixedSync, Indexing, Resampler,
    ResamplerConstructionError,
};
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
use tracing::{error, info, info_span};

use crate::playback::{
    constants::SAMPLE_BUFFER_CAPACITY, pipeline::channel_converter::AudioChannelConverter,
};

pub mod channel_converter;
// pub mod decoder;
// pub mod resampler;
// pub mod sink;

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
    pub configuration: Option<AudioPipelineConfiguration>,
    pub current_track: Option<AudioPipelineTrack>,
    pub resampler: Option<Fft<f32>>,
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
    sample_rate: u32,
    channels: u16,
    sample_buffer: VecDeque<f32>,
    decoded_sample_buffer: VecDeque<f32>,
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
            configuration: None,
            current_track: None,
            status: AudioPipelineStatus::Idle,
            resampler: None,
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
            .as_ref()
            .first_track_known_codec(TrackType::Audio)
            .ok_or_else(|| AudioPipelineError::MissingAudioTrack)?;

        let Some(CodecParameters::Audio(codec_parameters)) = track.codec_params.as_ref() else {
            return Err(AudioPipelineError::MissingCodecParameters.into());
        };

        let decoder =
            get_codecs().make_audio_decoder(&codec_parameters, &AudioDecoderOptions::default())?;

        let (Some(sample_rate), Some(channels)) = (
            codec_parameters.sample_rate,
            codec_parameters
                .channels
                .as_ref()
                .and_then(|channels| Some(channels.count())),
        ) else {
            return Err(AudioPipelineError::MissingCodecParameters);
        };

        info!("Now playing {:?}.", track_path);

        self.current_track = Some(AudioPipelineTrack {
            path: track_path,
            audio_track_id: track.id,
            demuxer,
            decoder,
            sample_rate: sample_rate,
            channels: channels as u16,
            sample_buffer: VecDeque::new(),
            decoded_sample_buffer: VecDeque::new(),
        });

        Ok(())
    }

    pub fn build_resampler(&mut self) -> Result<(), AudioPipelineError> {
        let Some(track) = self.current_track.as_ref() else {
            return Err(AudioPipelineError::MissingResamplerParameters);
        };

        let Some(configuration) = self.configuration.as_ref() else {
            return Err(AudioPipelineError::MissingResamplerParameters);
        };

        let (device_sample_rate, device_channels) =
            (configuration.sample_rate, configuration.channels);

        let resampling_channels = min(track.channels, device_channels);

        let resampler = Fft::<f32>::new(
            track.sample_rate as usize,
            device_sample_rate as usize,
            2048,
            2,
            resampling_channels.into(),
            FixedSync::Output,
        )?;

        info!(
            "Built resampler for transforming sample rate {}Hz to {}Hz with {} channels",
            track.sample_rate, device_sample_rate, resampling_channels
        );

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

        self.set_current_track(track_path)?;

        self.status = AudioPipelineStatus::ProducingSamples;

        Ok(())
    }

    pub fn update_pipeline_configuration(&mut self, configuration: AudioPipelineConfiguration) {
        info!(
            "Configuration set, output sample rate is {}Hz with {} channels",
            configuration.sample_rate, configuration.channels
        );

        self.configuration = Some(configuration);
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

        let Some(track) = audio_pipeline.current_track.as_ref() else {
            continue;
        };

        let Some(configuration) = audio_pipeline.configuration.as_ref() else {
            continue;
        };

        if track.sample_rate == configuration.sample_rate && audio_pipeline.resampler.is_some() {
            audio_pipeline.resampler = None;
        }

        if track.sample_rate != configuration.sample_rate && audio_pipeline.resampler.is_none() {
            match audio_pipeline.build_resampler() {
                Ok(_) => {}
                Err(error) => {
                    error!("Unable to create resampler for track and output device: {error}");

                    audio_pipeline.status = AudioPipelineStatus::Idle;
                    continue;
                }
            }
        }

        let Some(track) = audio_pipeline.current_track.as_mut() else {
            continue;
        };

        let Some(configuration) = audio_pipeline.configuration.as_mut() else {
            continue;
        };

        while !track.sample_buffer.is_empty() {
            let Some(sample) = track.sample_buffer.front() else {
                break;
            };

            match configuration.sample_buffer_producer.push(*sample) {
                Ok(_) => {
                    track.sample_buffer.pop_front();
                }
                Err(_) => {
                    // Sleep 50% of the time it takes to drain the buffer.
                    let sleep_duration_milliseconds =
                        ((SAMPLE_BUFFER_CAPACITY as f32 / configuration.channels as f32) * 1000.0
                            / configuration.sample_rate as f32)
                            * 0.5;

                    thread::sleep(Duration::from_millis(
                        sleep_duration_milliseconds.ceil() as u64
                    ));

                    break;
                }
            };
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
            info!("No more packets from demuxer.");
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

        // Resampling & Remixing
        let mut samples: Vec<f32> = Vec::new();

        generic_audio_buffer.copy_to_vec_interleaved(&mut samples);

        if track.channels > configuration.channels {
            match AudioChannelConverter::convert(&samples, track.channels, configuration.channels) {
                Ok(remixed_samples) => samples = remixed_samples,
                Err(error) => {
                    error!("Could not remix samples: {error}");

                    audio_pipeline.status = AudioPipelineStatus::Idle;

                    continue;
                }
            }
        }

        if let Some(resampler) = audio_pipeline.resampler.as_mut() {
            let mut decoded_samples: Vec<f32> = Vec::new();
            decoded_samples.extend(&track.decoded_sample_buffer);
            track.decoded_sample_buffer.clear();
            decoded_samples.extend(&samples);

            let input_frames = decoded_samples.len() / track.channels as usize;
            let mut input_frames_left = input_frames;
            let mut input_frames_next = resampler.input_frames_next();
            let input_adapter =
                InterleavedSlice::new(&mut decoded_samples, resampler.nbr_channels(), input_frames);

            let input_adapter = match input_adapter {
                Ok(adapter) => adapter,
                Err(error) => {
                    error!("Failed to create input adapter for resampling: {error}");

                    audio_pipeline.status = AudioPipelineStatus::Idle;
                    continue;
                }
            };

            let mut resampled_samples: Vec<f32> =
                vec![0.0; resampler.output_frames_max() * resampler.nbr_channels()];

            let output_frame_capacity = resampler.output_frames_max();
            let output_adapter = InterleavedSlice::new_mut(
                &mut resampled_samples,
                resampler.nbr_channels(),
                output_frame_capacity,
            );

            let mut output_adapter = match output_adapter {
                Ok(adapter) => adapter,
                Err(error) => {
                    error!("Failed to create output adapter for resampling: {error}");

                    audio_pipeline.status = AudioPipelineStatus::Idle;
                    continue;
                }
            };

            let mut indexing = Indexing {
                input_offset: 0,
                output_offset: 0,
                active_channels_mask: None,
                partial_len: None,
            };

            while input_frames_left >= input_frames_next {
                let resample_result = resampler.process_into_buffer(
                    &input_adapter,
                    &mut output_adapter,
                    Some(&indexing),
                );

                let (frames_read, frames_written) = match resample_result {
                    Ok(value) => value,
                    Err(error) => {
                        error!("Failed to resample chunk: {error}");

                        audio_pipeline.status = AudioPipelineStatus::Idle;
                        continue;
                    }
                };

                indexing.input_offset += frames_read;
                indexing.output_offset += frames_written;
                input_frames_left -= frames_read;
                input_frames_next = resampler.input_frames_next();
            }

            if (indexing.input_offset * resampler.nbr_channels()) < decoded_samples.len() {
                track
                    .decoded_sample_buffer
                    .extend(&decoded_samples[indexing.input_offset * resampler.nbr_channels()..]);
            }

            if track.channels < configuration.channels {
                match AudioChannelConverter::convert(
                    &resampled_samples,
                    track.channels,
                    configuration.channels,
                ) {
                    Ok(remixed_samples) => resampled_samples = remixed_samples,
                    Err(error) => {
                        error!("Could not remix samples: {error}");

                        audio_pipeline.status = AudioPipelineStatus::Idle;

                        continue;
                    }
                }
            }

            track.sample_buffer.extend(
                &resampled_samples[0..min(
                    indexing.output_offset * configuration.channels as usize,
                    resampled_samples.len(),
                )],
            );
        } else {
            if track.channels < configuration.channels {
                match AudioChannelConverter::convert(
                    &samples,
                    track.channels,
                    configuration.channels,
                ) {
                    Ok(remixed_samples) => samples = remixed_samples,
                    Err(error) => {
                        error!("Could not remix samples: {error}");

                        audio_pipeline.status = AudioPipelineStatus::Idle;

                        continue;
                    }
                }
            }

            track.sample_buffer.extend(samples);
        }
    });

    command_sender
}
