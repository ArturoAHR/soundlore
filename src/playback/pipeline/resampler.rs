use std::sync::Arc;
use std::{cmp::min, collections::VecDeque};

use rubato::{
    audioadapter_buffers::{direct::InterleavedSlice, SizeError},
    Fft, FixedSync, Indexing, ResampleError, Resampler, ResamplerConstructionError,
};
use thiserror::Error;
use tracing::{info, instrument};

use crate::playback::pipeline::AudioFormat;

/*
 * Ratio between chunk size and sub chunk size determines latency and quality, a higher ratio increases
 * quality and also latency, a lower ratio reduces quality but increases latency, recommended ratio is
 * 100 to 1000.
 *
 * Currently we are optimizing for highest quality.
 */
pub static RESAMPLER_CHUNK_SIZE: usize = 2048;
pub static RESAMPLER_SUB_CHUNK_SIZE: usize = 2;

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
            2048,
            2,
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

    pub fn resample(&mut self, input_samples: &[f32]) -> Result<Vec<f32>, AudioResamplerError> {
        let channels = self.resampler.nbr_channels();
        let mut samples: Vec<f32> =
            Vec::with_capacity(self.sample_buffer.len() + input_samples.len());

        samples.extend(self.sample_buffer.drain(..));
        samples.extend(input_samples);

        let input_frames = samples.len() / self.input.channels as usize;
        let mut input_frames_left = input_frames;
        let mut input_frames_next = self.resampler.input_frames_next();
        let input_adapter = InterleavedSlice::new(&mut samples, channels, input_frames)?;

        let mut resampled_samples: Vec<f32> =
            vec![0.0; self.resampler.output_frames_max() * channels];

        let output_frame_capacity = self.resampler.output_frames_max();
        let mut output_adapter =
            InterleavedSlice::new_mut(&mut resampled_samples, channels, output_frame_capacity)?;

        let mut indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: None,
        };

        while input_frames_left >= input_frames_next {
            let (frames_read, frames_written) = self.resampler.process_into_buffer(
                &input_adapter,
                &mut output_adapter,
                Some(&indexing),
            )?;

            indexing.input_offset += frames_read;
            indexing.output_offset += frames_written;
            input_frames_left -= frames_read;
            input_frames_next = self.resampler.input_frames_next();
        }

        if (indexing.input_offset * channels) < samples.len() {
            self.sample_buffer
                .extend(&samples[indexing.input_offset * channels..]);
        }

        let last_resampled_sample_index = min(
            indexing.output_offset * self.output.channels as usize,
            resampled_samples.len(),
        );

        let resampled_samples = resampled_samples[0..last_resampled_sample_index].to_vec();

        Ok(resampled_samples)
    }
}
