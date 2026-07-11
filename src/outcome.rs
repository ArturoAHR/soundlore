use iced::Task;
use tracing::instrument;

use crate::{
    app::{App, Message},
    error::AppError,
    event::Event::AttemptedPlayingTrack,
    playback::PlaybackControllerStatus,
    track::models::Track,
};

#[derive(Debug, Clone)]
pub enum Outcome {
    Playback(PlaybackOutcome),
}

#[derive(Debug, Clone)]
pub enum PlaybackOutcome {
    Resume,
    Pause,
    Stop,
    Play(String),
    Seek {
        timestamp: u64,
        post_seek_status: PlaybackControllerStatus,
    },
}

impl App {
    pub fn handle_outcome(&mut self, outcome: Outcome) -> Task<Message> {
        let outcome_task = match outcome {
            Outcome::Playback(outcome) => self.handle_playback_outcome(outcome),
        };

        match outcome_task {
            Ok(outcome_task) => outcome_task,
            Err(_error) => Task::none(), // TODO: Add error notification system
        }
    }

    #[instrument(skip(self))]
    fn handle_playback_outcome(
        &mut self,
        outcome: PlaybackOutcome,
    ) -> Result<Task<Message>, AppError> {
        match outcome {
            PlaybackOutcome::Resume => {
                self.playback_controller.resume()?;

                Ok(Task::none())
            }
            PlaybackOutcome::Stop => {
                self.playback_controller.stop()?;

                Ok(Task::none())
            }
            PlaybackOutcome::Pause => {
                self.playback_controller.pause()?;

                Ok(Task::none())
            }
            PlaybackOutcome::Seek {
                post_seek_status,
                timestamp,
            } => {
                self.playback_controller.seek(timestamp)?;

                match post_seek_status {
                    PlaybackControllerStatus::Playing => self.playback_controller.resume()?,
                    PlaybackControllerStatus::Stopped => self.playback_controller.pause()?,
                }

                Ok(Task::none())
            }

            PlaybackOutcome::Play(track_id) => {
                let track: Track = self
                    .tracks
                    .iter()
                    .find(|track| track.id == track_id)
                    .cloned()
                    .ok_or_else(|| AppError::TrackNotFound {
                        id: Some(track_id),
                        path: None,
                    })?
                    .to_owned();

                let event_tasks = self.broadcast(AttemptedPlayingTrack);

                self.playback_controller.play(track.clone())?;

                self.current_playing_track = Some(track);

                Ok(event_tasks)
            }
        }
    }
}
