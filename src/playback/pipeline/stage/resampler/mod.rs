use std::sync::Arc;
use std::{cmp::min, collections::VecDeque};

use rubato::{
    Fft, FixedSync, Indexing, ResampleError, Resampler, ResamplerConstructionError,
    audioadapter_buffers::{SizeError, direct::InterleavedSlice},
};
use thiserror::Error;
use tracing::{info, instrument, trace};

use crate::playback::constants::{RESAMPLER_CHUNK_SIZE, RESAMPLER_SUB_CHUNK_SIZE};
use crate::playback::pipeline::AudioFormat;

pub mod stage;

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
    pub channels: u16,

    pub status: AudioResamplerStatus,

    resampling_mode: ResamplingMode,
    total_samples_read: usize,
    total_samples_delivered: usize,

    resampler: Fft<f32>,
    /// Holds the samples that couldn't make it into a resampling chunk between calls.
    sample_buffer: VecDeque<f32>,
}

#[derive(Debug)]
pub enum AudioResamplerStatus {
    Resampling,
    Warmup(usize),
    Finished,
}

#[derive(Debug)]
pub enum ResamplingMode {
    Resample,
    Flush,
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

        let status = AudioResamplerStatus::Warmup(resampler.output_delay());

        info!("Built resampler",);

        Ok(Self {
            resampler,
            status,

            input: AudioFormat {
                sample_rate: input_sample_rate,
                channels: input_channels,
            },
            output: AudioFormat {
                sample_rate: output_sample_rate,
                channels: output_channels,
            },
            channels: resampling_channels,

            resampling_mode: ResamplingMode::Resample,
            total_samples_read: 0,
            total_samples_delivered: 0,

            sample_buffer: VecDeque::new(),
        })
    }

    /// Produces resampled samples, due to warmup samples initially output will be lower
    /// than average, excess samples get buffered for next call.
    ///
    /// On track end use `self.flush()` to get all the remaining buffered samples in the
    /// `AudioResampler` instance and within the `rubato::Resampler`.
    #[instrument(skip_all, level = "debug")]
    pub fn resample(&mut self, samples: &[f32]) -> Result<Vec<f32>, AudioResamplerError> {
        self.resampling_mode = ResamplingMode::Resample;

        let mut input_samples: Vec<f32> =
            Vec::with_capacity(self.sample_buffer.len() + samples.len());

        input_samples.extend(self.sample_buffer.drain(..));
        input_samples.extend(samples);

        let resampled_samples = self.process_samples(&input_samples)?;

        Ok(resampled_samples)
    }

    /// Flushes the contents inside the sample buffer in the instance and within the
    /// `rubato::Resampler` instance for the last remaining samples.
    #[instrument(skip_all, level = "debug")]
    pub fn flush(&mut self, samples: &[f32]) -> Result<Vec<f32>, AudioResamplerError> {
        self.resampling_mode = ResamplingMode::Flush;

        let mut input_samples: Vec<f32> =
            Vec::with_capacity(self.sample_buffer.len() + samples.len());

        input_samples.extend(samples);
        input_samples.extend(self.sample_buffer.drain(..));

        let resampled_samples = self.process_samples(&input_samples)?;

        self.status = AudioResamplerStatus::Finished;

        Ok(resampled_samples)
    }

    #[instrument(skip_all, level = "trace")]
    fn process_samples(&mut self, samples: &[f32]) -> Result<Vec<f32>, AudioResamplerError> {
        self.total_samples_read += samples.len();

        let input_frames = samples.len() / self.resampler.nbr_channels();

        let mut indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: self.get_indexing_partial_len(input_frames),
        };

        let mut resampled_samples = Vec::new();

        let input_adapter =
            InterleavedSlice::new(samples, self.resampler.nbr_channels(), input_frames)?;

        let mut input_frames_left = input_frames;
        let mut output_frames_left = if matches!(self.resampling_mode, ResamplingMode::Flush) {
            // Gets remaining output frames we need to output based on how many samples were read and how
            // many were delivered in the output sample rate.
            ((self.total_samples_read as f64 * self.resampler.resample_ratio()).round() as usize
                - self.total_samples_delivered)
                / self.resampler.nbr_channels()
        } else {
            0
        };

        let mut output_samples: Vec<f32> =
            vec![0.0; self.resampler.output_frames_max() * self.resampler.nbr_channels()];

        while self.should_keep_resampling(input_frames_left, output_frames_left) {
            let mut output_adapter = InterleavedSlice::new_mut(
                &mut output_samples,
                self.resampler.nbr_channels(),
                self.resampler.output_frames_max(),
            )?;

            let (frames_read, frames_written) = self.resampler.process_into_buffer(
                &input_adapter,
                &mut output_adapter,
                Some(&indexing),
            )?;

            indexing.input_offset += frames_read;
            input_frames_left -= min(frames_read, input_frames_left);

            let (output_samples_range_start, output_samples_range_end) =
                self.get_output_samples_range(frames_written, output_frames_left);

            if output_samples_range_start <= output_samples_range_end {
                resampled_samples
                    .extend(&output_samples[output_samples_range_start..output_samples_range_end]);
            }

            if let AudioResamplerStatus::Warmup(mut warmup_frames) = self.status {
                warmup_frames -= min(
                    min(output_samples_range_start, output_samples_range_end)
                        / self.resampler.nbr_channels(),
                    warmup_frames,
                );

                if warmup_frames == 0 {
                    self.status = AudioResamplerStatus::Resampling;
                } else {
                    self.status = AudioResamplerStatus::Warmup(warmup_frames);

                    continue;
                }
            }

            output_frames_left -= min(frames_written, output_frames_left);

            // After no more input frames are left we start processing silence to output the remaining
            // resampled samples.
            if input_frames_left == 0 && matches!(self.resampling_mode, ResamplingMode::Flush) {
                indexing.input_offset = 0;
                indexing.partial_len = Some(0);
            }
        }

        if matches!(self.resampling_mode, ResamplingMode::Resample)
            && indexing.input_offset < samples.len()
        {
            self.sample_buffer
                .extend(&samples[indexing.input_offset * self.resampler.nbr_channels()..]);

            self.total_samples_read -= self.sample_buffer.len();
        }

        self.total_samples_delivered += resampled_samples.len();
        trace!("Resampled samples: {}", resampled_samples.len());

        Ok(resampled_samples)
    }

    /// Gets the output range we should extract from the output buffer, skips warmup samples.
    fn get_output_samples_range(
        &self,
        frames_written: usize,
        output_frames_left: usize,
    ) -> (usize, usize) {
        let mut output_frames = min(frames_written, self.resampler.output_frames_max());

        if matches!(self.resampling_mode, ResamplingMode::Flush) {
            output_frames = min(output_frames, output_frames_left);
        }

        let last_resampled_sample_index = output_frames * self.resampler.nbr_channels();

        let first_resampled_sample_index =
            if let AudioResamplerStatus::Warmup(warmup_frames) = self.status {
                // Skip resampler warmup frames
                warmup_frames * self.resampler.nbr_channels()
            } else {
                0
            };

        (first_resampled_sample_index, last_resampled_sample_index)
    }

    /// If we are resampling normally we just output until the input frames are not enough to do another
    /// resampled chunk, otherwise we just output until there are no more output frames left.
    fn should_keep_resampling(&self, input_frames_left: usize, output_frames_left: usize) -> bool {
        match self.resampling_mode {
            ResamplingMode::Flush => output_frames_left != 0,
            ResamplingMode::Resample => input_frames_left >= self.resampler.input_frames_next(),
        }
    }

    /// If we are resampling normally the indexing should only work with full input chunks, otherwise
    /// if we are flushing it should work with any amount of frames we've got.
    fn get_indexing_partial_len(&self, input_frames: usize) -> Option<usize> {
        match self.resampling_mode {
            ResamplingMode::Flush => Some(input_frames),
            ResamplingMode::Resample => None,
        }
    }
}
