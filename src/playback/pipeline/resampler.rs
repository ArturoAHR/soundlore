use std::sync::Arc;
use std::{cmp::min, collections::VecDeque};

use rubato::{
    audioadapter_buffers::{direct::InterleavedSlice, SizeError},
    Fft, FixedSync, Indexing, ResampleError, Resampler, ResamplerConstructionError,
};
use thiserror::Error;
use tracing::{info, instrument};

use crate::playback::constants::{RESAMPLER_CHUNK_SIZE, RESAMPLER_SUB_CHUNK_SIZE};
use crate::playback::pipeline::AudioFormat;

#[derive(Debug, Error, Clone)]
pub enum AudioResamplerError {
    #[error("failed to build resampler: {0}")]
    ResamplerBuildFailed(Arc<ResamplerConstructionError>),

    #[error("failed to build resampler adapter: {0}")]
    ResamplerAdapterBuildFailed(Arc<SizeError>),

    #[error("resampler error: {0}")]
    Resampler(Arc<ResampleError>),
}

impl From<ResamplerConstructionError> for AudioResamplerError {
    fn from(error: ResamplerConstructionError) -> Self {
        Self::ResamplerBuildFailed(Arc::new(error))
    }
}

impl From<SizeError> for AudioResamplerError {
    fn from(error: SizeError) -> Self {
        Self::ResamplerAdapterBuildFailed(Arc::new(error))
    }
}

impl From<ResampleError> for AudioResamplerError {
    fn from(error: ResampleError) -> Self {
        Self::Resampler(Arc::new(error))
    }
}

#[derive(Debug)]
pub struct AudioResampler {
    pub input: AudioFormat,
    pub output: AudioFormat,

    resampler: Fft<f32>,
    /// Holds the samples that couldn't make it into a resampling chunk between calls.
    sample_buffer: VecDeque<f32>,
}

impl AudioResampler {
    #[instrument(level = "debug")]
    pub fn build(
        input_sample_rate: u32,
        input_channels: u16,
        output_sample_rate: u32,
        output_channels: u16,
    ) -> Result<Self, AudioResamplerError> {
        let resampling_channels = min(input_channels, output_channels);

        let resampler = Fft::<f32>::new(
            input_sample_rate as usize,
            output_sample_rate as usize,
            RESAMPLER_CHUNK_SIZE,
            RESAMPLER_SUB_CHUNK_SIZE,
            resampling_channels.into(),
            FixedSync::Output,
        )?;

        info!("Built resampler",);

        Ok(Self {
            resampler,
            input: AudioFormat {
                sample_rate: input_sample_rate,
                channels: input_channels,
            },
            output: AudioFormat {
                sample_rate: output_sample_rate,
                channels: output_channels,
            },
            sample_buffer: VecDeque::new(),
        })
    }

    pub fn resample(&mut self, samples: &[f32]) -> Result<Vec<f32>, AudioResamplerError> {
        let channels = self.resampler.nbr_channels();
        let mut input_samples: Vec<f32> =
            Vec::with_capacity(self.sample_buffer.len() + samples.len());

        input_samples.extend(self.sample_buffer.drain(..));
        input_samples.extend(samples);

        let input_frames = input_samples.len() / channels as usize;
        let mut input_frames_left = input_frames;
        let mut input_frames_next = self.resampler.input_frames_next();
        let input_adapter = InterleavedSlice::new(&mut input_samples, channels, input_frames)?;

        let output_frame_capacity = self.resampler.output_frames_max();

        let mut output_samples: Vec<f32> = vec![0.0; output_frame_capacity * channels];

        let mut indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: None,
        };

        let mut resampled_samples = Vec::new();
        while input_frames_left >= input_frames_next {
            let mut output_adapter =
                InterleavedSlice::new_mut(&mut output_samples, channels, output_frame_capacity)?;

            let (frames_read, frames_written) = self.resampler.process_into_buffer(
                &input_adapter,
                &mut output_adapter,
                Some(&indexing),
            )?;

            indexing.input_offset += frames_read;
            input_frames_left -= frames_read;
            input_frames_next = self.resampler.input_frames_next();

            let last_resampled_sample_index = min(
                frames_written * channels as usize,
                output_frame_capacity * channels,
            );

            resampled_samples.extend(&output_samples[0..last_resampled_sample_index]);
        }

        if (indexing.input_offset * channels) < input_samples.len() {
            self.sample_buffer
                .extend(&input_samples[indexing.input_offset * channels..]);
        }

        Ok(resampled_samples)
    }
}
