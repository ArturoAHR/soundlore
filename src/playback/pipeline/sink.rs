use std::collections::VecDeque;

use rtrb::Producer;
use thiserror::Error;

use crate::playback::constants::SAMPLE_BUFFER_CAPACITY;

#[derive(Debug, Error, Clone)]
pub enum AudioSinkError {
    #[error("the audio engine ring buffer is full")]
    FullRingBuffer,
    #[error("awaiting for the ring buffer to clear")]
    AwaitingBufferClear,
}

pub struct AudioSink {
    status: AudioSinkStatus,

    audio_engine_producer: Producer<f32>,
    sample_buffer: VecDeque<f32>,
}

#[derive(PartialEq)]
pub enum AudioSinkStatus {
    Writing,
    Clearing,
}

impl AudioSink {
    pub fn new(audio_engine_producer: Producer<f32>) -> Self {
        Self {
            audio_engine_producer,

            sample_buffer: VecDeque::new(),

            status: AudioSinkStatus::Writing,
        }
    }

    pub fn write(&mut self) -> Result<(), AudioSinkError> {
        if self.status == AudioSinkStatus::Clearing {
            if self.audio_engine_producer.slots() != SAMPLE_BUFFER_CAPACITY {
                return Err(AudioSinkError::AwaitingBufferClear);
            }

            self.status = AudioSinkStatus::Writing;
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

    pub fn buffer(&mut self, samples: &[f32]) {
        self.sample_buffer.extend(samples);
    }

    pub fn clear(&mut self) {
        self.sample_buffer.clear();

        self.status = AudioSinkStatus::Clearing;
    }
}
