use tracing::{error, instrument};

use crate::playback::{
    PlaybackController, PlaybackControllerError, PlaybackControllerStatus,
    engine::PlaybackEngineStatus, pipeline::thread::AudioPipelineThreadEvent,
};

impl PlaybackController {
    /// Polls the audio pipeline event receiver
    ///
    /// # Errors
    /// Returns an error if the pipeline event receiver fails its poll attempt
    #[instrument(skip(self), err)]
    pub fn handle_audio_pipeline_event(
        &mut self,
        event: &AudioPipelineThreadEvent,
    ) -> Result<(), PlaybackControllerError> {
        match event {
            AudioPipelineThreadEvent::StartedAudioPipeline => {
                #[allow(clippy::collapsible_if)]
                if matches!(*self.playback_engine.status(), PlaybackEngineStatus::Paused) {
                    if let Err(error) = self.playback_engine.play() {
                        error!(
                            "Could not start playback engine after decode thread started producing samples: {error}"
                        );
                    }
                }

                self.status = PlaybackControllerStatus::Playing;
            }
            AudioPipelineThreadEvent::StoppedAudioPipeline => {
                #[allow(clippy::collapsible_if)]
                if matches!(
                    *self.playback_engine.status(),
                    PlaybackEngineStatus::Playing
                ) {
                    if let Err(error) = self.playback_engine.pause() {
                        error!(
                            "Could not pause playback engine after decode thread stopped producing samples: {error}"
                        );
                    }
                }

                self.status = PlaybackControllerStatus::Stopped;
            }

            _ => {}
        }

        Ok(())
    }
}
