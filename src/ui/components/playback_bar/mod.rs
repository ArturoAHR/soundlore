use iced::{
    Element, Renderer, Task,
    widget::{button, container, row, slider},
};
use tracing::instrument;

use crate::{
    event::Event,
    outcome::PlaybackOutcome,
    playback::PlaybackControllerStatus,
    track::models::Track,
    ui::{
        theme::Theme,
        widgets::icons::{self, icon},
    },
};

pub mod handler;

#[derive(Debug)]
pub struct PlaybackBar {
    current_position: f64,
    pub current_position_generation_threshold: u64,

    status: PlaybackBarStatus,
}

#[derive(Debug)]
pub enum PlaybackBarStatus {
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub enum Message {
    Pause,
    Resume,
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

#[derive(Debug)]
pub struct PlaybackBarEventContext {
    pub playback_engine_generation: u64,
}

impl PlaybackBar {
    pub fn new() -> Self {
        Self {
            status: PlaybackBarStatus::Playing,
            current_position: 0.0,
            current_position_generation_threshold: 0,
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn update(
        &mut self,
        event: Message,
        ctx: PlaybackBarUpdateContext,
    ) -> (Task<Message>, Vec<Outcome>) {
        match event {
            Message::Scrubbed(position) => {
                self.current_position = position;

                self.current_position_generation_threshold = ctx.playback_engine_generation;

                let mut outcomes = Vec::new();
                if PlaybackControllerStatus::Playing == *ctx.playback_controller_status {
                    outcomes.push(Outcome::Playback(PlaybackOutcome::Pause));
                };

                (Task::none(), outcomes)
            }
            Message::PlaybackProgressed(position) => {
                self.current_position = position;

                (Task::none(), Vec::new())
            }
            Message::Seeked => {
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
            Message::Resume => {
                self.status = PlaybackBarStatus::Playing;

                (
                    Task::none(),
                    vec![Outcome::Playback(PlaybackOutcome::Resume)],
                )
            }
            Message::Pause => {
                self.status = PlaybackBarStatus::Paused;

                (
                    Task::none(),
                    vec![Outcome::Playback(PlaybackOutcome::Pause)],
                )
            }
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, event: &Event, ctx: PlaybackBarEventContext) -> Task<Message> {
        let task = Task::none();

        match event {
            Event::AttemptedPlayingTrack => {
                self.current_position_generation_threshold = ctx.playback_engine_generation
            }
            _ => {}
        }

        task
    }

    pub fn view<'a>(
        &'a self,
        ctx: PlaybackBarViewContext,
    ) -> Element<'a, Message, Theme, Renderer> {
        let mut total_frames = 1.0;
        let mut current_position = 0.0;

        if let Some(track) = ctx.current_playing_track {
            total_frames = track.frames as f64;
            current_position = self.current_position;
        }

        let play_button = match self.status {
            PlaybackBarStatus::Paused => button(icon(icons::PLAY)).on_press(Message::Resume),
            PlaybackBarStatus::Playing => button(icon(icons::PAUSE)).on_press(Message::Pause),
        };

        container(row![
            play_button,
            slider(0.0..=total_frames, current_position, Message::Scrubbed)
                .on_release(Message::Seeked)
        ])
        .into()
    }
}
