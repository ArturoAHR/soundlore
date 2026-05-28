use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, DefaultStreamConfigError, OutputCallbackInfo, Stream,
};
use thiserror::Error;
use tracing::error;

use crate::error::AppError;

#[derive(Debug, Error, Clone)]
pub enum PlaybackEngineError {
    #[error("output device not found")]
    OutputDeviceNotFound,
    #[error("default stream configuration error")]
    DefaultStreamConfiguration(#[from] DefaultStreamConfigError),
    #[error("build stream failed")]
    StreamBuildFailed(#[from] BuildStreamError),
}

pub struct PlaybackEngine {
    pub stream: Stream,
}

impl PlaybackEngine {
    pub fn build() -> Result<Self, AppError> {
        let host = default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| PlaybackEngineError::OutputDeviceNotFound)?;
        let config = device
            .default_output_config()
            .map_err(PlaybackEngineError::DefaultStreamConfiguration)?;

        let sample_rate = config.sample_rate() as f32;
        let channels = config.channels() as usize;
        let mut sample_clock = 0f32;

        let mut next_sample = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    for frame in data.chunks_mut(channels) {
                        let value = next_sample();
                        for sample in frame.iter_mut() {
                            *sample = value
                        }
                    }
                },
                |error| error!("stream error: {error}"),
                None,
            )
            .map_err(PlaybackEngineError::StreamBuildFailed)?;

        Ok(Self { stream })
    }
}
