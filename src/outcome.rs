use iced::Task;
use tracing::instrument;

use crate::{
    app::{App, Event},
    error::AppError,
    message::Message,
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
    Play(Track),
    Seek {
        timestamp: u64,
        post_seek_status: PlaybackControllerStatus,
    },
}

impl App {
    pub fn handle_outcome(&mut self, outcome: Message<Outcome>) -> Task<Message<Event>> {
        let outcome_task = match outcome.payload {
            Outcome::Playback(ref playback_outcome) => {
                self.handle_playback_outcome(outcome.new_from(playback_outcome.clone()))
            }
        };

        match outcome_task {
            Ok(outcome_task) => outcome_task,
            Err(_error) => Task::none(), // TODO: Add error notification system
        }
    }

    #[instrument(skip(self))]
    fn handle_playback_outcome(
        &mut self,
        outcome: Message<PlaybackOutcome>,
    ) -> Result<Task<Message<Event>>, AppError> {
        match outcome.payload {
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
            PlaybackOutcome::Play(track) => {
                self.playback_controller.play(track.clone())?;

                self.current_playing_track = Some(track);

                Ok(Task::none())
            }
        }
    }
}
