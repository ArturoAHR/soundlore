use crate::playback::pipeline::{
    AudioPipelineError,
    config::AudioTrackPipelineConfiguration,
    stage::{
        AudioPipelineBaseStage, AudioPipelineProcessStage, AudioPipelineSamples,
        channel_converter::AudioChannelConverter,
    },
};

pub struct AudioPipelineChannelConverterStage {
    position: AudioPipelineChannelConverterStagePosition,
}

pub enum AudioPipelineChannelConverterStagePosition {
    BeforeResampling,
    AfterResampling,
}

impl AudioPipelineChannelConverterStage {
    pub fn new(position: AudioPipelineChannelConverterStagePosition) -> Self {
        Self { position }
    }
}

impl AudioPipelineBaseStage<AudioTrackPipelineConfiguration>
    for AudioPipelineChannelConverterStage
{
    fn is_enabled(&self, configuration: &AudioTrackPipelineConfiguration) -> bool {
        let input_channels = configuration.track.channels;
        let output_channels = i64::from(configuration.output.channels);

        if input_channels == output_channels {
            return false;
        }

        match self.position {
            AudioPipelineChannelConverterStagePosition::AfterResampling => {
                input_channels < output_channels
            }
            AudioPipelineChannelConverterStagePosition::BeforeResampling => {
                input_channels > output_channels
            }
        }
    }
}

impl AudioPipelineProcessStage<AudioTrackPipelineConfiguration>
    for AudioPipelineChannelConverterStage
{
    fn process_stage(
        &mut self,
        configuration: &AudioTrackPipelineConfiguration,
        samples: AudioPipelineSamples,
    ) -> Result<AudioPipelineSamples, AudioPipelineError> {
        let input_channels = configuration.track.channels as u16;
        let output_channels = configuration.output.channels;

        let converted_samples =
            AudioChannelConverter::convert(samples.as_ref(), input_channels, output_channels)?;

        Ok(samples.new_like(converted_samples))
    }
}
