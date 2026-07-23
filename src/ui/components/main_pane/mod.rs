use iced::{
    Element, Length, Renderer, Task, alignment,
    widget::{Space, container, text},
};
use iced_palace::widget::ellipsized_text;
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::instrument;

use crate::{
    event::Event,
    outcome::PlaybackOutcome,
    track::models::{Track, TrackId},
    ui::{
        theme::Theme,
        utils::label::format_duration,
        widgets::{
            icons::{self, icon},
            table::{column, table},
        },
    },
};

pub mod handler;

#[derive(Debug)]
pub struct MainPane {
    pub selected_track_ids: FxHashSet<i64>,
    pub displayed_track_ids: Vec<TrackId>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetDisplayedTracks(Vec<TrackId>),
    TrackRowDoubleClicked(TrackId),
    TrackRowSelected(FxHashSet<TrackId>),
    ColumnHeaderCellClicked(TrackTableColumn),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrackTableColumn {
    NowPlaying,
    Title,
    Artist,
    Duration,
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
            selected_track_ids: FxHashSet::default(),
            displayed_track_ids: Vec::new(),
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Message) -> (Task<Message>, Vec<Outcome>) {
        let task = Task::none();
        let mut outcomes = Vec::new();

        match event {
            Message::SetDisplayedTracks(displayed_track_ids) => {
                self.displayed_track_ids = displayed_track_ids;
            }
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
        tracks: &'a FxHashMap<TrackId, Track>,
        displayed_track_ids: &Vec<TrackId>,
        current_playing_track_id: Option<&TrackId>,
    ) -> Element<'a, Message, Theme, Renderer> {
        let current_playing_track_id = current_playing_track_id.copied().unwrap_or(-1);

        let columns = vec![
            column(TrackTableColumn::NowPlaying, None, move |track: &Track| {
                if track.id == current_playing_track_id {
                    icon(icons::PLAY)
                } else {
                    Space::new().into()
                }
            })
            .width(30.0),
            column(
                TrackTableColumn::Artist,
                Some(text("Artist").into()),
                |track: &Track| {
                    ellipsized_text(track.artist.as_deref().unwrap_or("Unknown"))
                        .wrapping(text::Wrapping::None)
                },
            )
            .width(200.0)
            .resizable(true),
            column(
                TrackTableColumn::Title,
                Some(text("Title").into()),
                |track: &Track| {
                    ellipsized_text(track.title.as_deref().unwrap_or("Untitled"))
                        .wrapping(text::Wrapping::None)
                },
            )
            .width(200.0)
            .resizable(true),
            column(
                TrackTableColumn::Duration,
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
            table(
                columns,
                displayed_track_ids
                    .iter()
                    .filter_map(|track_id| tracks.get(track_id))
                    .collect(),
            )
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
