use thiserror::Error;

pub mod stage;

#[derive(Debug, Error, Clone)]
pub enum AudioChannelConverterError {
    #[error("unsupported conversion: {0} to {1}.")]
    UnsupportedConversion(u16, u16),
}

pub struct AudioChannelConverter {}

impl AudioChannelConverter {
    pub fn convert(
        samples: &[f32],
        input_channels: u16,
        output_channels: u16,
    ) -> Result<Vec<f32>, AudioChannelConverterError> {
        if input_channels == output_channels {
            return Ok(Vec::from(samples));
        }

        if input_channels == 1 && output_channels == 2 {
            return Ok(samples
                .iter()
                .zip(samples)
                .flat_map(|(l, r)| [l, r])
                .cloned()
                .collect());
        }

        if input_channels == 2 && output_channels == 1 {
            let left_samples = samples.iter().step_by(2);
            let right_samples = samples.iter().skip(1).step_by(2);

            return Ok(left_samples
                .zip(right_samples)
                .map(|(l, r)| (l + r) / 2.0)
                .collect());
        }

        return Err(AudioChannelConverterError::UnsupportedConversion(
            input_channels,
            output_channels,
        ));
    }
}
