use iced::{widget::text, Element, Renderer, Task};
use tracing::instrument;

use crate::ui::theme::Theme;

#[derive(Debug)]
pub struct MainPane {}

#[derive(Debug, Clone)]
pub enum Event {}

#[derive(Debug, Clone)]
pub enum Outcome {}

impl MainPane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Event) -> (Task<Event>, Option<Outcome>) {
        (Task::none(), None)
    }

    pub fn view<'a>(&'a self, theme: &Theme) -> Element<'a, Event, Theme, Renderer> {
        text("Main Pane").into()
    }
}
