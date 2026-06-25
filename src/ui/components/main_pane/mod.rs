use iced::{
    Element, Renderer, Task,
    widget::{Column, button, text},
};
use tracing::instrument;

use crate::{message::Message, outcome::PlaybackOutcome, track::models::Track, ui::theme::Theme};

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

#[derive(Debug)]
pub struct MainPaneViewContext<'a> {
    pub theme: &'a Theme,
    pub tracks: &'a Vec<Track>,
}

#[derive(Debug)]
pub struct MainPaneUpdateContext {}

impl MainPane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(
        &mut self,
        event: Message<Event>,
    ) -> (Task<Message<Event>>, Vec<Message<Outcome>>) {
        match event.payload {
            Event::TrackSelected(ref track) => (
                Task::none(),
                vec![event.new_from(Outcome::Playback(PlaybackOutcome::Play(track.clone())))],
            ),
        }
    }

    pub fn view<'a>(
        &'a self,
        ctx: MainPaneViewContext,
    ) -> Element<'a, Message<Event>, Theme, Renderer> {
        let track_rows: Vec<Element<Message<Event>, Theme, Renderer>> = ctx
            .tracks
            .iter()
            .map(|track| {
                button(text(format!(
                    "{} - {}",
                    track.artist.clone().unwrap_or("Unknown".to_owned()),
                    track.title.clone().unwrap_or("Untitled".to_owned())
                )))
                .on_press(Message::new(Event::TrackSelected(track.to_owned())))
                .into()
            })
            .collect();

        Column::with_children(track_rows).into()

        // text("Main Pane").into()
    }
}
