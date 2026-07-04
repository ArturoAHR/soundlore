use iced::{
    Element, Length, Renderer, Task,
    widget::{self, button, container, scrollable, text},
};
use tracing::instrument;

use crate::{
    event::Event,
    outcome::PlaybackOutcome,
    track::models::Track,
    ui::{
        theme::Theme,
        widgets::table::{column, table},
    },
};

pub mod handler;

#[derive(Debug)]
pub struct MainPane {}

#[derive(Debug, Clone)]
pub enum Message {
    TrackSelected(Track),
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
        match event {
            Message::TrackSelected(track) => (
                Task::none(),
                vec![Outcome::Playback(PlaybackOutcome::Play(track))],
            ),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, _event: &Event) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(
        &'a self,
        ctx: MainPaneViewContext<'a>,
    ) -> Element<'a, Message, Theme, Renderer> {
        let track_rows: Vec<Element<Message, Theme, Renderer>> = ctx
            .tracks
            .iter()
            .map(|track| {
                button(text(format!(
                    "{} - {}",
                    track.artist.clone().unwrap_or("Unknown".to_owned()),
                    track.title.clone().unwrap_or("Untitled".to_owned())
                )))
                .on_press(Message::TrackSelected(track.to_owned()))
                .into()
            })
            .collect();

        let _columns = scrollable(
            widget::Column::with_children(track_rows)
                .width(Length::Fill)
                .height(Length::Fill),
        );

        let columns = vec![
            column(Some(text("Title").into()), |track: &Track| {
                text(track.title.clone().unwrap_or("Untitled".to_owned()))
            }),
            column(Some(text("Artist").into()), |track: &Track| {
                text(track.artist.clone().unwrap_or("Unknown".to_owned()))
            }),
            column(Some(text("").into()), |track: &Track| {
                text(track.artist.clone().unwrap_or("Unknown".to_owned()))
            }),
        ];

        container(table(columns, &ctx.tracks))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.palette.surface.into()),
                ..container::Style::default()
            })
            .into()
    }
}
