use iced::{Element, Renderer, Task, widget::slider};
use tracing::instrument;

use crate::{
    message::Message,
    outcome::PlaybackOutcome,
    playback::PlaybackControllerStatus,
    track::models::Track,
    ui::{components::playback_bar::PlaybackBarStatus::Playing, theme::Theme},
};

pub mod handler;

#[derive(Debug)]
pub struct PlaybackBar {
    current_position: f64,
    pub seek_generation_threshold: u64,

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

#[derive(Debug)]
pub struct PlaybackBarViewContext<'a> {
    pub theme: &'a Theme,
    pub current_playing_track: &'a Option<Track>,
}

#[derive(Debug)]
pub struct PlaybackBarUpdateContext<'a> {
    pub playback_controller_status: &'a PlaybackControllerStatus,
    pub playback_engine_generation: u64,
}

impl PlaybackBar {
    pub fn new() -> Self {
        Self {
            status: Playing,
            current_position: 0.0,
            seek_generation_threshold: 0,
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn update(
        &mut self,
        event: Event,
        ctx: PlaybackBarUpdateContext,
    ) -> (Task<Event>, Vec<Outcome>) {
        match event {
            Event::Scrubbed(position) => {
                self.current_position = position;

                self.seek_generation_threshold = ctx.playback_engine_generation;

                let mut outcomes = Vec::new();
                if PlaybackControllerStatus::Playing == *ctx.playback_controller_status {
                    outcomes.push(Outcome::Playback(PlaybackOutcome::Pause));
                };

                (Task::none(), outcomes)
            }
            Event::PlaybackProgressed(position) => {
                self.current_position = position;

                (Task::none(), Vec::new())
            }
            Event::Seeked => {
                let pre_seek_status = match self.status {
                    PlaybackBarStatus::Playing => PlaybackControllerStatus::Playing,
                    PlaybackBarStatus::Paused => PlaybackControllerStatus::Stopped,
                };

                (
                    Task::none(),
                    vec![Outcome::Playback(PlaybackOutcome::Seek {
                        timestamp: self.current_position.round() as u64,
                        post_seek_status: pre_seek_status,
                    })],
                )
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        ctx: PlaybackBarViewContext,
    ) -> Element<'a, Message<Event>, Theme, Renderer> {
        let mut total_frames = 1.0;
        let mut current_position = 0.0;

        if let Some(track) = ctx.current_playing_track {
            total_frames = track.frames as f64;
            current_position = self.current_position;
        }

        slider(
            0.0..=total_frames,
            current_position,
            Message::from_payload(Event::Scrubbed),
        )
        .on_release(Message::new(Event::Seeked))
        .into()
    }
}
