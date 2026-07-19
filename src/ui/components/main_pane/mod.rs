use std::collections::HashSet;

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
pub struct MainPane {
    pub selected_track_ids: HashSet<TableIdentifier>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TrackRowDoubleClicked(TableIdentifier),
    TrackRowSelected(HashSet<TableIdentifier>),
    ColumnHeaderCellClicked(TableIdentifier),
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Playback(PlaybackOutcome),
}

#[derive(Debug)]
pub struct MainPaneUpdateContext {}

impl MainPane {
    pub fn new() -> Self {
        Self {
            selected_track_ids: HashSet::new(),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Message) -> (Task<Message>, Vec<Outcome>) {
        let task = Task::none();
        let mut outcomes = Vec::new();

        match event {
            Message::TrackRowDoubleClicked(track_id) => {
                outcomes.push(Outcome::Playback(PlaybackOutcome::Play(track_id)));
            }
            Message::TrackRowSelected(selected_track_ids) => {
                self.selected_track_ids = selected_track_ids.into_iter().collect();
            }
            Message::ColumnHeaderCellClicked(_column_id) => {}
        }

        (task, outcomes)
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, event: &Event) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(
        &'a self,
        _theme: &'a Theme,
        tracks: &'a [Track],
    ) -> Element<'a, Message, Theme, Renderer> {
        let columns = vec![
            column(
                "artist".to_owned(),
                Some(text("Artist").into()),
                |track: &Track| {
                    ellipsized_text(track.artist.clone().unwrap_or_else(|| "Unknown".to_owned()))
                        .wrapping(text::Wrapping::None)
                },
            )
            .width(200.0)
            .resizable(true),
            column(
                "title".to_owned(),
                Some(text("Title").into()),
                |track: &Track| {
                    ellipsized_text(track.title.clone().unwrap_or_else(|| "Untitled".to_owned()))
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
            table(columns, tracks)
                .selected_rows(&self.selected_track_ids)
                .on_row_select(Message::TrackRowSelected)
                .on_row_double_click(Message::TrackRowDoubleClicked)
                .on_header_cell_click(Message::ColumnHeaderCellClicked),
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

impl Default for MainPane {
    fn default() -> Self {
        Self::new()
    }
}
