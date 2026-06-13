use symphonia::core::formats::SeekMode;
use tracing::{debug, instrument};

use crate::playback::pipeline::{
    AudioPipelineError,
    config::AudioTrackPipelineConfiguration,
    stage::{
        AudioPipelineBaseStage, AudioPipelineSamples, AudioPipelineSourceStage,
        AudioPipelineStageCommandOutcome,
        decoder::{AudioDecoder, AudioDecoderStatus},
    },
    thread::AudioPipelineThreadCommand,
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
    #[instrument(err, skip(self, _configuration), level = "debug")]
    fn handle_command(
        &mut self,
        _configuration: &AudioTrackPipelineConfiguration,
        command: &AudioPipelineThreadCommand,
    ) -> Result<Option<AudioPipelineStageCommandOutcome>, AudioPipelineError> {
        match command {
            AudioPipelineThreadCommand::Seek(timestamp) => {
                let seeked_to = self.decoder.seek(*timestamp, SeekMode::Coarse)?;

                let new_timestamp = seeked_to.actual_ts.get() as u64;

                debug!(
                    seek_timestamp = timestamp,
                    actual_timestamp = new_timestamp,
                    "Decoder seek."
                );

                Ok(Some(AudioPipelineStageCommandOutcome::SeekedTo(
                    new_timestamp,
                )))
            }
            AudioPipelineThreadCommand::Stop => {
                let _ = self.decoder.seek(0, SeekMode::Accurate)?;

                Ok(Some(AudioPipelineStageCommandOutcome::SeekedTo(0)))
            }
            _ => Ok(None),
        }
    }

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
