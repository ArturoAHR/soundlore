use iced::{
    Element, Length, Renderer, Task, alignment,
    widget::{container, text},
};
use iced_palace::widget::ellipsized_text;
use tracing::instrument;

use crate::{
    event::Event,
    outcome::PlaybackOutcome,
    track::models::Track,
    ui::{
        theme::Theme,
        utils::label::format_duration,
        widgets::table::{column, state::TableIdentifier, table},
    },
};

pub mod handler;

#[derive(Debug)]
pub struct MainPane {}

#[derive(Debug, Clone)]
pub enum Message {
    TrackRowDoubleClicked(TableIdentifier),
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Playback(PlaybackOutcome),
}

#[derive(Debug)]
pub struct MainPaneViewContext<'a> {
    pub theme: &'a Theme,
    pub tracks: &'a Vec<Track>,
}

#[derive(Debug)]
pub struct MainPaneUpdateContext {}

impl MainPane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Message) -> (Task<Message>, Vec<Outcome>) {
        let task = Task::none();
        let mut outcomes = Vec::new();

        match event {
            Message::TrackRowDoubleClicked(track_id) => {
                outcomes.push(Outcome::Playback(PlaybackOutcome::Play(track_id)))
            }
        }

        return (task, outcomes);
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, _event: &Event) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(
        &'a self,
        ctx: MainPaneViewContext<'a>,
    ) -> Element<'a, Message, Theme, Renderer> {
        let columns = vec![
            column(
                "artist".to_owned(),
                Some(text("Artist").into()),
                |track: &Track| {
                    ellipsized_text(track.artist.clone().unwrap_or("Unknown".to_owned()))
                        .wrapping(text::Wrapping::None)
                },
            )
            .width(200.0)
            .resizable(true),
            column(
                "title".to_owned(),
                Some(text("Title").into()),
                |track: &Track| {
                    ellipsized_text(track.title.clone().unwrap_or("Untitled".to_owned()))
                        .wrapping(text::Wrapping::None)
                },
            )
            .width(200.0)
            .resizable(true),
            column(
                "duration".to_owned(),
                Some(text("Duration").into()),
                |track: &Track| {
                    ellipsized_text(format_duration(
                        track.frames as u64 / track.sample_rate as u64,
                    ))
                    .wrapping(text::Wrapping::None)
                },
            )
            .width(50.0)
            .resizable(true)
            .align_x(alignment::Horizontal::Right),
        ];

        container(
            table(columns, &ctx.tracks)
                .on_row_double_click(|track_id| Message::TrackRowDoubleClicked(track_id)),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.palette.surface.into()),
            ..container::Style::default()
        })
        .into()
    }
}
