use iced::{
    Element, Renderer, Task,
    widget::{slider, text},
};
use tracing::instrument;

use crate::{
    outcome::PlaybackOutcome,
    playback::{PlaybackController, PlaybackControllerStatus},
    track::models::Track,
    ui::{components::playback_bar::PlaybackBarStatus::Playing, theme::Theme},
};

pub mod handler;

#[derive(Debug)]
pub struct PlaybackBar {
    current_position: f64,

    status: PlaybackBarStatus,
}

#[derive(Debug)]
pub enum PlaybackBarStatus {
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub enum Event {
    Scrubbed(f64),
    Seeked,
    PlaybackProgressed(f64),
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Playback(PlaybackOutcome),
}

impl PlaybackBar {
    pub fn new() -> Self {
        Self {
            status: Playing,
            current_position: 0.0,
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn update(
        &mut self,
        event: Event,
        playback_controller_status: &PlaybackControllerStatus,
    ) -> (Task<Event>, Option<Outcome>) {
        match event {
            Event::Scrubbed(position) => {
                self.current_position = position;

                let outcome = match playback_controller_status {
                    PlaybackControllerStatus::Playing => {
                        Some(Outcome::Playback(PlaybackOutcome::Pause))
                    }
                    PlaybackControllerStatus::Stopped => None,
                };

                (Task::none(), outcome)
            }
            Event::PlaybackProgressed(position) => {
                self.current_position = position;

                (Task::none(), None)
            }
            Event::Seeked => {
                let pre_seek_status = match self.status {
                    PlaybackBarStatus::Playing => PlaybackControllerStatus::Playing,
                    PlaybackBarStatus::Paused => PlaybackControllerStatus::Stopped,
                };

                (
                    Task::none(),
                    Some(Outcome::Playback(PlaybackOutcome::Seek {
                        timestamp: self.current_position.round() as u64,
                        post_seek_status: pre_seek_status,
                    })),
                )
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        _theme: &Theme,
        track: &Option<Track>,
    ) -> Element<'a, Event, Theme, Renderer> {
        let mut total_frames = 1.0;
        let mut current_position = 0.0;

        if let Some(track) = track {
            total_frames = track.frames as f64;
            current_position = self.current_position;
        }

        slider(0.0..=total_frames, current_position, Event::Scrubbed)
            .on_release(Event::Seeked)
            .into()
    }
}
