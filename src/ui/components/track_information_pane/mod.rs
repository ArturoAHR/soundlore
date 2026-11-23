use iced::{
    Element, Length, Renderer, Task, alignment,
    widget::{Space, center, column, container, scrollable, text},
};
use rustc_hash::FxHashMap;
use tracing::instrument;

use crate::{
    app::PlaybackOwner,
    event::Event,
    track::models::{Track, TrackId},
    ui::{
        components::track_information_pane::widgets::track_information,
        theme::{Theme, catalog},
    },
};

pub mod handler;
pub mod widgets;

#[derive(Debug, Default)]
pub struct TrackInformationPane {
    playing_track_id: Option<TrackId>,
}

#[derive(Debug, Clone)]
#[expect(clippy::empty_enums, reason = "placeholder")]
pub enum Message {}

#[derive(Debug, Clone)]
#[expect(clippy::empty_enums, reason = "placeholder")]
pub enum Outcome {}

impl TrackInformationPane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Message) -> (Task<Message>, Vec<Outcome>) {
        (Task::none(), vec![])
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, event: &Event, playback_owner: &PlaybackOwner) -> Task<Message> {
        let task = Task::none();

        if !matches!(playback_owner, PlaybackOwner::PlaybackBar) {
            return task;
        }

        #[allow(clippy::single_match)]
        match event {
            Event::ActiveTrackChanged(track_id) => {
                self.playing_track_id = *track_id;
            }
            _ => {}
        }

        task
    }

    pub fn view<'a>(
        &'a self,
        theme: &Theme,
        tracks: &'a FxHashMap<TrackId, Track>,
    ) -> Element<'a, Message, Theme, Renderer> {
        let track_information_labels = self
            .playing_track_id
            .and_then(|playing_track_id| tracks.get(&playing_track_id))
            .map_or_else(
                || Space::new().into(),
                |track| track_information(theme, track),
            );

        container(
            scrollable(
                column![
                    track_information_labels,
                    center(text("No image").color(theme.palette.text_muted))
                        .width(300)
                        .height(300)
                        .style(catalog::container::track_thumbnail)
                ]
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .spacing(theme.sizes.space.xl)
                .padding(theme.sizes.space.xl),
            )
            .style(catalog::scrollable::pane),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .style(catalog::container::background_surface_raised)
        .into()
    }
}
