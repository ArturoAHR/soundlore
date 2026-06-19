use tracing::instrument;

use std::cmp::min;

use crate::playback::pipeline::{
    AudioPipelineCommand, AudioPipelineError,
    config::AudioTrackPipelineConfiguration,
    stage::{
        AudioPipelineBaseStage, AudioPipelineCommandOutcome, AudioPipelineProcessStage,
        AudioPipelineSamples, resampler::AudioResampler,
    },
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
    fn handle_command(
        &mut self,
        configuration: &AudioTrackPipelineConfiguration,
        command: &AudioPipelineCommand,
    ) -> Result<Option<AudioPipelineCommandOutcome>, AudioPipelineError> {
        match command {
            AudioPipelineCommand::Seek(_) | AudioPipelineCommand::Stop => {
                self.rebuild_resampler(configuration)?;

                Ok(None)
            }
        }
    }

    fn is_enabled(&self, configuration: &AudioTrackPipelineConfiguration) -> bool {
        let input_sample_rate = configuration.track.sample_rate;
        let output_sample_rate = configuration.output.sample_rate as i64;

        input_sample_rate != output_sample_rate
    }
}

impl AudioPipelineProcessStage<AudioTrackPipelineConfiguration> for AudioPipelineResamplerStage {
    fn process_stage(
        &mut self,
        configuration: &AudioTrackPipelineConfiguration,
        samples: AudioPipelineSamples,
    ) -> Result<AudioPipelineSamples, AudioPipelineError> {
        // If output device changes in a significant way, we need to rebuild.
        if configuration.output.sample_rate != self.resampler.output.sample_rate
            || min(
                configuration.track.channels as u16,
                configuration.output.channels,
            ) != self.resampler.channels
        {
            self.rebuild_resampler(configuration)?;
        }

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

impl AudioPipelineResamplerStage {
    #[instrument(skip_all, level = "debug")]
    fn rebuild_resampler(
        &mut self,
        configuration: &AudioTrackPipelineConfiguration,
    ) -> Result<(), AudioPipelineError> {
        let input_sample_rate = configuration.track.sample_rate as u32;
        let input_channels = configuration.track.channels as u16;
        let output_sample_rate = configuration.output.sample_rate;
        let output_channels = configuration.output.channels;

        let new_resampler = AudioResampler::build(
            input_sample_rate,
            input_channels,
            output_sample_rate,
            output_channels,
        )?;

        self.resampler = new_resampler;

        Ok(())
    }
}
