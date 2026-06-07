use crate::playback::pipeline::{
    AudioPipelineError,
    config::AudioTrackPipelineConfiguration,
    stage::{
        AudioPipelineBaseStage, AudioPipelineSamples, AudioPipelineSourceStage,
        decoder::{AudioDecoder, AudioDecoderStatus},
    },
};

pub struct AudioPipelineDecoderStage {
    decoder: AudioDecoder,
}

impl AudioPipelineDecoderStage {
    pub fn new(decoder: AudioDecoder) -> Self {
        Self { decoder }
    }
}

impl AudioPipelineBaseStage<AudioTrackPipelineConfiguration> for AudioPipelineDecoderStage {
    fn is_enabled(&self, _: &AudioTrackPipelineConfiguration) -> bool {
        !matches!(self.decoder.status, AudioDecoderStatus::Finished)
    }
}

impl AudioPipelineSourceStage<AudioTrackPipelineConfiguration> for AudioPipelineDecoderStage {
    fn process_stage(
        &mut self,
        configuration: &AudioTrackPipelineConfiguration,
    ) -> Result<AudioPipelineSamples, AudioPipelineError> {
        let decoded_samples = self.decoder.decode()?;

        for event in self.decoder.pending_events.drain(..) {
            configuration.event_emitter.emit(event);
        }

        Ok(decoded_samples)
    }
}
