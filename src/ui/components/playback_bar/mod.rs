use iced::{Element, Renderer, Task, widget::text};
use tracing::instrument;

use crate::ui::theme::Theme;

#[derive(Debug)]
pub struct PlaybackBar {}

#[derive(Debug, Clone)]
pub enum Event {}

#[derive(Debug, Clone)]
pub enum Outcome {}

impl PlaybackBar {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Event) -> (Task<Event>, Option<Outcome>) {
        (Task::none(), None)
    }

    pub fn view<'a>(&'a self, theme: &Theme) -> Element<'a, Event, Theme, Renderer> {
        text("Playback Bar").into()
    }
}
