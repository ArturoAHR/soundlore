use std::{
    collections::VecDeque,
    sync::{Arc, atomic::Ordering},
};

use rtrb::Producer;
use thiserror::Error;
use tracing::instrument;

use crate::playback::{GenerationCounter, constants::SAMPLE_BUFFER_CAPACITY};

#[derive(Debug, Error, Clone)]
pub enum AudioSinkError {
    #[error("the audio engine ring buffer is full")]
    FullRingBuffer,
    #[error("awaiting for the ring buffer to clear")]
    AwaitingBufferClear,
}

pub struct AudioSink {
    audio_engine_producer: Producer<f32>,
    sample_buffer: VecDeque<f32>,

    generation_counter: Arc<GenerationCounter>,
}

impl AudioSink {
    pub fn new(
        audio_engine_producer: Producer<f32>,
        generation_counter: Arc<GenerationCounter>,
    ) -> Self {
        Self {
            audio_engine_producer,
            generation_counter,

            sample_buffer: VecDeque::new(),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn write(&mut self) -> Result<(), AudioSinkError> {
        let audio_engine_generation = self.generation_counter.audio_engine.load(Ordering::Acquire);
        let audio_pipeline_generation = self
            .generation_counter
            .audio_pipeline
            .load(Ordering::Relaxed);

        if audio_engine_generation != audio_pipeline_generation {
            return Err(AudioSinkError::AwaitingBufferClear);
        }

        while !self.sample_buffer.is_empty() {
            let Some(sample) = self.sample_buffer.front() else {
                break;
            };

            // TODO: Implement push_partial_slice instead
            self.audio_engine_producer
                .push(*sample)
                .map_err(|_| AudioSinkError::FullRingBuffer)?;

            self.sample_buffer.pop_front();
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.sample_buffer.is_empty()
    }

    pub fn is_engine_buffer_empty(&self) -> bool {
        self.audio_engine_producer.slots() == SAMPLE_BUFFER_CAPACITY
    }

    pub fn buffer(&mut self, samples: &[f32]) {
        self.sample_buffer.extend(samples);
    }

    pub fn clear(&mut self) {
        self.sample_buffer.clear();
    }
}
