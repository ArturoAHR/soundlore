use iced::{
    Alignment, Element, Font, Length, Padding, Renderer, Task,
    widget::{Space, button, column, container, row, slider, text},
};
use tracing::instrument;

use crate::{
    event::Event,
    outcome::PlaybackOutcome,
    playback::PlaybackControllerStatus,
    track::{
        models::Track,
        utils::{get_track_duration_label, get_track_label},
    },
    ui::{
        components::playback_bar::widgets::volume_bar,
        theme::Theme,
        utils::seconds_to_timestamp,
        widgets::icons::{self, icon},
    },
};

pub mod handler;
pub mod widgets;

#[derive(Debug)]
pub struct PlaybackBar {
    current_position: f64,
    pub current_position_generation_threshold: u64,

    volume_percentage: u8,
    muted: bool,

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
    ChangeVolumePercentage(u8),
    MutePlayback,
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

            muted: false,
            volume_percentage: 100,
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn update(
        &mut self,
        event: Message,
        ctx: PlaybackBarUpdateContext,
    ) -> (Task<Message>, Vec<Outcome>) {
        let task = Task::none();
        let mut outcomes = Vec::new();

        match event {
            Message::Scrubbed(position) => {
                self.current_position = position;

                self.current_position_generation_threshold = ctx.playback_engine_generation;

                if PlaybackControllerStatus::Playing == *ctx.playback_controller_status {
                    outcomes.push(Outcome::Playback(PlaybackOutcome::Pause));
                };
            }
            Message::PlaybackProgressed(position) => {
                self.current_position = position;
            }
            Message::Seeked => {
                let pre_seek_status = match self.status {
                    PlaybackBarStatus::Playing => PlaybackControllerStatus::Playing,
                    PlaybackBarStatus::Paused => PlaybackControllerStatus::Stopped,
                };

                outcomes = vec![Outcome::Playback(PlaybackOutcome::Seek {
                    timestamp: self.current_position.round() as u64,
                    post_seek_status: pre_seek_status,
                })];
            }
            Message::Resume => {
                self.status = PlaybackBarStatus::Playing;

                outcomes = vec![Outcome::Playback(PlaybackOutcome::Resume)];
            }
            Message::Pause => {
                self.status = PlaybackBarStatus::Paused;

                outcomes = vec![Outcome::Playback(PlaybackOutcome::Pause)];
            }
            // TODO: Add playback outcome to change volume
            Message::ChangeVolumePercentage(volume_percentage) => {
                self.volume_percentage = volume_percentage;
            }
            // TODO: Add playback outcome to change volume
            Message::MutePlayback => {
                self.muted = !self.muted;
            }
        };

        (task, outcomes)
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, event: &Event, ctx: PlaybackBarEventContext) -> Task<Message> {
        let task = Task::none();

        match event {
            Event::AttemptedPlayingTrack => {
                self.status = PlaybackBarStatus::Playing;

                self.current_position_generation_threshold = ctx.playback_engine_generation;
                self.current_position = 0.0;
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

        let mut track_name_label = String::new();
        let mut track_duration_timestamp = "0:00".to_owned();
        let mut current_position_timestamp = "0:00".to_owned();

        if let Some(track) = ctx.current_playing_track {
            total_frames = track.frames as f64;
            current_position = self.current_position;
            track_name_label = get_track_label(track);

            track_duration_timestamp = get_track_duration_label(track);
            current_position_timestamp = seconds_to_timestamp(
                ((current_position / total_frames) * total_frames / track.sample_rate as f64)
                    .floor() as u64,
            )
        }

        let play_previous = button(icon(icons::PLAY_PREVIOUS));
        let play_next = button(icon(icons::PLAY_NEXT));
        let play_button = match self.status {
            PlaybackBarStatus::Paused => button(icon(icons::PLAY)).on_press(Message::Resume),
            PlaybackBarStatus::Playing => button(icon(icons::PAUSE)).on_press(Message::Pause),
        };

        let current_time_label = format!(
            "{} / {}",
            current_position_timestamp, track_duration_timestamp
        );

        container(
            row![
                row![play_previous, play_button, play_next],
                column![
                    row![
                        text(track_name_label),
                        Space::new().width(Length::Fill),
                        text(current_time_label).font(Font::MONOSPACE)
                    ],
                    slider(0.0..=total_frames, current_position, Message::Scrubbed)
                        .on_release(Message::Seeked)
                ]
                .spacing(10.0),
                volume_bar(self.volume_percentage, self.muted)
            ]
            .align_y(Alignment::Center)
            .spacing(20.0),
        )
        .height(Length::Fixed(90.0))
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .padding(Padding::from(15.0))
        .style(|theme: &Theme| container::Style {
            background: Some(theme.palette.surface_raised.into()),
            ..container::Style::default()
        })
        .into()
    }
}
