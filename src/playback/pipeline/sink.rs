use std::collections::VecDeque;

use rtrb::Producer;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum AudioSinkError {
    #[error("the audio engine ring buffer is full")]
    FullRingBuffer,
}

pub struct AudioSink {
    audio_engine_producer: Producer<f32>,
    sample_buffer: VecDeque<f32>,
}

impl AudioSink {
    pub fn new(audio_engine_producer: Producer<f32>) -> Self {
        Self {
            audio_engine_producer,

            sample_buffer: VecDeque::new(),
        }
    }

    pub fn write(&mut self) -> Result<(), AudioSinkError> {
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

    pub fn buffer(&mut self, samples: &[f32]) {
        self.sample_buffer.extend(samples);
    }
}
