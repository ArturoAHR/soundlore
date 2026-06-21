use std::{
    ops::ControlFlow,
    path::Path,
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};

use rtrb::Producer;
use tracing::{error, info_span, warn};

use crate::{
    playback::{
        GenerationCounter,
        constants::SAMPLE_BUFFER_CAPACITY,
        pipeline::{
            AudioFormat, AudioPipelineError, AudioPipelineStatus, builder::AudioPipelineBuilder,
            sink::AudioSinkError, stage::decoder::AudioDecoderError,
        },
    },
    track::models::Track,
};

#[derive(Debug)]
pub enum AudioPipelineThreadCommand {
    Play(Track),
    Pause,
    Resume,
    PlayNext,
    PlayPrevious,
    Seek(u64),
    Stop,
    ChangeNextTrack(Track),
    ChangeOutput {
        output: AudioFormat,
        audio_engine_producer: Producer<f32>,
    },
    Exit,
}

#[derive(Debug, Clone)]
pub enum AudioPipelineThreadEvent {
    DecodingFinished,
    Exited,
    TrackFinished,
    StartedAudioPipeline,
    StoppedAudioPipeline,
    ActiveTrackChanged { track_id: String },
    UnexpectedError(AudioPipelineError),
}

pub fn spawn_audio_pipeline_thread(
    samples_played_timestamp_offset: Arc<AtomicU64>,
    generation_counter: Arc<GenerationCounter>,
) -> (
    std::thread::JoinHandle<()>,
    std::sync::mpsc::Sender<AudioPipelineThreadCommand>,
    std::sync::mpsc::Receiver<AudioPipelineThreadEvent>,
) {
    let (command_sender, command_receiver) = std::sync::mpsc::channel();
    let (event_sender, event_receiver) = std::sync::mpsc::channel();

    let audio_pipeline_thread_handle = std::thread::spawn(move || {
        let span = info_span!(parent: None, "audio_decoding_loop");
        let _guard = span.entered();

        let audio_pipeline_builder = AudioPipelineBuilder::new(
            command_receiver,
            event_sender,
            samples_played_timestamp_offset,
            generation_counter,
        );
        let Ok(mut audio_pipeline) = audio_pipeline_builder.build() else {
            error!("Closing audio pipeline thread due to builder error");

            return;
        };

        loop {
            match audio_pipeline.process() {
                Ok(ControlFlow::Continue(_)) => {}
                Ok(ControlFlow::Break(_)) => {
                    audio_pipeline
                        .configuration
                        .event_emitter
                        .emit(AudioPipelineThreadEvent::Exited);

                    break;
                }
                Err(AudioPipelineError::Sink(AudioSinkError::FullRingBuffer)) => {
                    let output_format = &audio_pipeline.configuration.output;

                    let sleep_duration_milliseconds =
                        ((SAMPLE_BUFFER_CAPACITY as f32 / output_format.channels as f32) * 1000.0
                            / output_format.sample_rate as f32)
                            * 0.5;

                    std::thread::sleep(Duration::from_millis(
                        sleep_duration_milliseconds.ceil() as u64
                    ));
                }
                Err(AudioPipelineError::Sink(AudioSinkError::AwaitingBufferClear)) => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(AudioPipelineError::AwaitingEndOfTrack) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(AudioPipelineError::Decoder(AudioDecoderError::RecoverableDecoderError(
                    error,
                ))) => {
                    warn!("recoverable audio pipeline error: {error}");
                }
                Err(error) => {
                    let current_track =
                        audio_pipeline
                            .audio_track_pipelines
                            .get(0)
                            .map(|audio_track_pipeline| {
                                Path::new(&audio_track_pipeline.configuration.track.file_path)
                                    .file_name()
                                    .unwrap_or(
                                        audio_track_pipeline.configuration.track.file_path.as_ref(),
                                    )
                                    .to_owned()
                            });

                    audio_pipeline.set_status(AudioPipelineStatus::Paused);

                    error!(current_track = ?current_track, "audio pipeline error: {error}");

                    audio_pipeline
                        .configuration
                        .event_emitter
                        .emit(AudioPipelineThreadEvent::UnexpectedError(error));
                }
            }
        }
    });

    (audio_pipeline_thread_handle, command_sender, event_receiver)
}
