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
    pub status: AudioResamplerStatus,

    total_frames: u64,
    delivered_frames: u64,
    channels: usize,

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

impl AudioResampler {
    #[instrument(level = "debug")]
    pub fn build(
        input_sample_rate: u32,
        input_channels: u16,
        output_sample_rate: u32,
        output_channels: u16,
        track_frames: u64,
        starting_frame: Option<u64>,
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

        let resample_ratio = resampler.resample_ratio();
        let channels = resampler.nbr_channels();

        let total_frames = (track_frames as f64 * resample_ratio).ceil() as u64;
        // println!("TEST - total_frames = {total_frames}, track_frames = {track_frames}, resample_ratio = {resample_ratio}");
        let delivered_frames = (starting_frame.unwrap_or(0) as f64 * resample_ratio).ceil() as u64;

        info!("Built resampler",);

        Ok(Self {
            resampler,
            status,
            total_frames,
            delivered_frames,
            channels,

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

    /// Produces resampled samples, due to warmup samples initially output will be lower
    /// than average, excess samples get buffered for next call.
    ///
    /// On track end use `self.flush()` to get all the remaining buffered samples in the
    /// `AudioResampler` instance and within the `rubato::Resampler`.
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

        // Resample loop
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

            let mut first_real_resampled_sample_index = 0;

            // Skip resampler warmup frames
            if let AudioResamplerStatus::Warmup(warmup_frames) = self.status {
                let first_real_resampled_frame =
                    min(warmup_frames, last_resampled_sample_index / channels);

                // If frames resampled are less than warmup frames, skip, otherwise start resampled samples output.
                if first_real_resampled_frame >= warmup_frames {
                    self.status = AudioResamplerStatus::Resampling;

                    first_real_resampled_sample_index = first_real_resampled_frame * channels;
                } else {
                    self.status =
                        AudioResamplerStatus::Warmup(warmup_frames - first_real_resampled_frame);

                    continue;
                }
            }

            resampled_samples.extend(
                &output_samples[first_real_resampled_sample_index..last_resampled_sample_index],
            );
        }

        // Any samples that couldn't make it into a resampling chunk get buffered for later
        if (indexing.input_offset * channels) < input_samples.len() {
            self.sample_buffer
                .extend(&input_samples[indexing.input_offset * channels..]);
        }

        self.delivered_frames += (resampled_samples.len() / channels) as u64;

        Ok(resampled_samples)
    }

    /// Flushes the contents inside the sample buffer in the instance and within the
    /// `rubato::Resampler` instance for the last remaining samples.
    pub fn flush(&mut self, samples: &[f32]) -> Result<Vec<f32>, AudioResamplerError> {
        if self.total_frames < self.delivered_frames {
            return Ok(Vec::new());
        }

        let channels = self.channels;

        let mut input_samples: Vec<f32> =
            Vec::with_capacity(self.sample_buffer.len() + samples.len());

        input_samples.extend(samples);
        input_samples.extend(self.sample_buffer.drain(..));

        let input_frames = input_samples.len() / channels as usize;

        // println!(
        //     "TEST - total_frames: {}, delivered_frames: {}",
        //     self.total_frames, self.delivered_frames
        // );

        let mut indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: Some(input_frames),
        };

        let output_frame_capacity = self.resampler.output_frames_max();
        let mut resampled_samples = Vec::new();

        while self.delivered_frames <= self.total_frames {
            let input_adapter = InterleavedSlice::new(&mut input_samples, channels, input_frames)?;
            let mut output_samples: Vec<f32> = vec![0.0; output_frame_capacity * channels];
            let mut output_adapter =
                InterleavedSlice::new_mut(&mut output_samples, channels, output_frame_capacity)?;

            let (frames_read, frames_written) = self.resampler.process_into_buffer(
                &input_adapter,
                &mut output_adapter,
                Some(&indexing),
            )?;

            resampled_samples.extend(
                &output_samples[0..min(
                    frames_written,
                    (self.total_frames - min(self.delivered_frames, self.total_frames)) as usize,
                ) * channels],
            );

            indexing.input_offset += frames_read;
            self.delivered_frames += frames_written as u64;

            // println!("TEST - frames_read: {frames_read}, frames_written: {frames_written}");
            // println!(
            //     "TEST - total_frames: {}, delivered_frames: {}",
            //     self.total_frames, self.delivered_frames
            // );

            if indexing.input_offset >= input_frames {
                indexing.input_offset = 0;
                indexing.partial_len = Some(0);
            }
        }

        self.status = AudioResamplerStatus::Finished;

        Ok(resampled_samples)
    }
}
