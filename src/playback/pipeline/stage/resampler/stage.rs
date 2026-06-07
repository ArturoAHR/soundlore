use crate::playback::pipeline::{
    config::AudioTrackPipelineConfiguration,
    stage::{
        resampler::AudioResampler, AudioPipelineBaseStage, AudioPipelineProcessStage,
        AudioPipelineSamples,
    },
    AudioPipelineError,
};

pub struct AudioPipelineResamplerStage {
    resampler: AudioResampler,
}

impl AudioPipelineResamplerStage {
    pub fn new(resampler: AudioResampler) -> Self {
        Self { resampler }
    }
}

impl AudioPipelineBaseStage<AudioTrackPipelineConfiguration> for AudioPipelineResamplerStage {
    fn is_enabled(&self, configuration: &AudioTrackPipelineConfiguration) -> bool {
        let input_sample_rate = configuration.track.sample_rate;
        let output_sample_rate = configuration.output.sample_rate as i64;

        input_sample_rate != output_sample_rate
    }
}

impl AudioPipelineProcessStage<AudioTrackPipelineConfiguration> for AudioPipelineResamplerStage {
    fn process_stage(
        &mut self,
        _configuration: &AudioTrackPipelineConfiguration,
        samples: AudioPipelineSamples,
    ) -> Result<AudioPipelineSamples, AudioPipelineError> {
        match samples {
            AudioPipelineSamples::Chunk(samples) => {
                let resampled_samples = self.resampler.resample(samples.as_ref())?;

                Ok(AudioPipelineSamples::Chunk(resampled_samples))
            }
            AudioPipelineSamples::End(samples) => {
                let resampled_samples = self.resampler.flush(samples.as_ref())?;

                Ok(AudioPipelineSamples::End(resampled_samples))
            }
        }
    }
}
