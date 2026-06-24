use iced::{
    Element, Renderer, Task,
    widget::{Column, button, text},
};
use tracing::instrument;

use crate::{outcome::PlaybackOutcome, track::models::Track, ui::theme::Theme};

pub mod handler;

#[derive(Debug)]
pub struct MainPane {}

#[derive(Debug, Clone)]
pub enum Event {
    TrackSelected(Track),
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Playback(PlaybackOutcome),
}

impl MainPane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Event) -> (Task<Event>, Vec<Outcome>) {
        match event {
            Event::TrackSelected(track) => (
                Task::none(),
                vec![Outcome::Playback(PlaybackOutcome::Play(track))],
            ),
        }
    }

    pub fn view<'a>(
        &'a self,
        _theme: &Theme,
        tracks: &Vec<Track>,
    ) -> Element<'a, Event, Theme, Renderer> {
        let track_rows: Vec<Element<Event, Theme, Renderer>> = tracks
            .iter()
            .map(|track| {
                button(text(format!(
                    "{} - {}",
                    track.artist.clone().unwrap_or("Unknown".to_owned()),
                    track.title.clone().unwrap_or("Untitled".to_owned())
                )))
                .on_press(Event::TrackSelected(track.to_owned()))
                .into()
            })
            .collect();

        Column::with_children(track_rows).into()

        // text("Main Pane").into()
    }
}
