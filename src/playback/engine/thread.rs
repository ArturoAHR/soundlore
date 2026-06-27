use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
};

use cpal::{
    Device, FromSample, OutputCallbackInfo, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, StreamTrait},
};
use rtrb::Consumer;
use tracing::{error, warn};

use crate::playback::{
    GenerationCounter,
    engine::{PlaybackEngineError, constants::OUTPUT_VOLUME_MULTIPLIER},
};

#[cfg(test)]
#[path = "thread_test.rs"]
mod thread_test;

pub struct AudioEngineStreamBuildArguments {
    pub device: Device,
    pub config: StreamConfig,
    pub sample_buffer_consumer: Consumer<f32>,
    pub samples_played: Arc<AtomicU64>,
    pub track_start_timestamp: Arc<AtomicI64>,
    pub samples_played_timestamp_offset: Arc<AtomicU64>,
    pub generation_counter: Arc<GenerationCounter>,
    pub paused: Arc<AtomicBool>,
}

pub fn build_output_stream<T>(
    arguments: AudioEngineStreamBuildArguments,
) -> Result<Stream, PlaybackEngineError>
where
    T: SizedSample + FromSample<f32>,
{
    let config = arguments.config;
    let device = arguments.device;

    let mut data_processor = AudioEngineDataProcessor {
        sample_buffer_consumer: arguments.sample_buffer_consumer,
        samples_played: arguments.samples_played,
        track_start_timestamp: arguments.track_start_timestamp,
        samples_played_timestamp_offset: arguments.samples_played_timestamp_offset,
        generation_counter: arguments.generation_counter,
        paused: arguments.paused,
    };

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [T], info: &OutputCallbackInfo| {
            data_processor.process(data, info);
        },
        |error| error!("stream error: {error}"),
        None,
    )?;

    if let Err(error) = stream.play() {
        warn!(
            error = %error,
            "Failed to start playing with created output stream, this could be due to the output device not supporting the command and may not affect playback."
        );
    }

    Ok(stream)
}

struct AudioEngineDataProcessor {
    sample_buffer_consumer: Consumer<f32>,
    samples_played: Arc<AtomicU64>,
    track_start_timestamp: Arc<AtomicI64>,
    samples_played_timestamp_offset: Arc<AtomicU64>,
    generation_counter: Arc<GenerationCounter>,
    paused: Arc<AtomicBool>,
}

impl AudioEngineDataProcessor {
    pub fn process<T>(&mut self, data: &mut [T], _: &OutputCallbackInfo)
    where
        T: SizedSample + FromSample<f32>,
    {
        let audio_pipeline_generation = self
            .generation_counter
            .audio_pipeline
            .load(Ordering::Acquire);
        let audio_engine_generation = self.generation_counter.audio_engine.load(Ordering::Relaxed);

        if audio_engine_generation != audio_pipeline_generation {
            let start_timestamp = self.samples_played.load(Ordering::Relaxed) as i64
                - self.samples_played_timestamp_offset.load(Ordering::Relaxed) as i64;

            self.track_start_timestamp
                .store(start_timestamp, Ordering::Release);

            // Clear ring buffer
            while self.sample_buffer_consumer.pop().is_ok() {}

            self.generation_counter
                .audio_engine
                .store(audio_pipeline_generation, Ordering::Release);
        }

        let mut samples_played = 0;
        let paused = self.paused.load(Ordering::Relaxed);
        for slot in data.iter_mut() {
            if !paused {
                if let Ok(sample) = self.sample_buffer_consumer.pop() {
                    // TODO: Remove this once we have volume built
                    *slot = T::from_sample_(sample * OUTPUT_VOLUME_MULTIPLIER);
                    samples_played += 1;

                    continue;
                }
            }

            *slot = T::from_sample_(0.0);
        }

        self.samples_played
            .fetch_add(samples_played, Ordering::Relaxed);
    }
}
