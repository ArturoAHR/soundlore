use std::sync::mpsc::TryRecvError;

use tracing::{error, instrument};

use crate::playback::{
    PlaybackController, PlaybackControllerError, PlaybackControllerStatus,
    engine::PlaybackEngineStatus, error::get_audio_controller_error_message,
    pipeline::thread::AudioPipelineThreadEvent,
};

#[derive(Debug, Clone)]
pub enum PlaybackControllerEvent {
    StartedPlayback,
    StoppedPlayback,
    EndOfTrack,
    PlayingTrack { track_id: String },
    UnexpectedError { message: String },
}

impl PlaybackController {
    #[instrument(skip(self), err)]
    pub fn poll_event(
        &mut self,
    ) -> Result<Option<PlaybackControllerEvent>, PlaybackControllerError> {
        let audio_pipeline_event = match self.audio_pipeline_event_receiver.try_recv() {
            Ok(event) => event,
            Err(TryRecvError::Empty) => return Ok(None),
            Err(error) => {
                return Err(
                    PlaybackControllerError::AudioPipelineEventReceiveAttemptFailed(
                        error.to_string(),
                    ),
                );
            }
        };

        match audio_pipeline_event {
            AudioPipelineThreadEvent::StartedAudioPipeline => {
                if *self.playback_engine.status() == PlaybackEngineStatus::Paused {
                    if let Err(error) = self.playback_engine.play() {
                        error!(
                            "Could not start playback engine after decode thread started producing samples: {error}"
                        );
                    }
                }

                self.status = PlaybackControllerStatus::Playing;

                Ok(Some(PlaybackControllerEvent::StartedPlayback))
            }
            AudioPipelineThreadEvent::StoppedAudioPipeline => {
                if *self.playback_engine.status() == PlaybackEngineStatus::Playing {
                    if let Err(error) = self.playback_engine.pause() {
                        error!(
                            "Could not pause playback engine after decode thread stopped producing samples: {error}"
                        );
                    }
                }

                self.status = PlaybackControllerStatus::Stopped;

                Ok(Some(PlaybackControllerEvent::StoppedPlayback))
            }
            AudioPipelineThreadEvent::TrackFinished => {
                Ok(Some(PlaybackControllerEvent::EndOfTrack))
            }
            AudioPipelineThreadEvent::UnexpectedError(error) => {
                Ok(Some(PlaybackControllerEvent::UnexpectedError {
                    message: get_audio_controller_error_message(error),
                }))
            }
            AudioPipelineThreadEvent::ActiveTrackChanged { track_id } => {
                Ok(Some(PlaybackControllerEvent::PlayingTrack { track_id }))
            }
            _ => Ok(None),
        }
    }
}
