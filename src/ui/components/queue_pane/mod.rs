use iced::{Element, Renderer, Task, widget::text};
use tracing::instrument;

use crate::ui::theme::Theme;

pub mod handler;

#[derive(Debug)]
pub struct QueuePane {}

#[derive(Debug, Clone)]
pub enum Event {}

#[derive(Debug, Clone)]
pub enum Outcome {}

impl QueuePane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Event) -> (Task<Event>, Vec<Outcome>) {
        (Task::none(), vec![])
    }

    pub fn view<'a>(&'a self, _theme: &Theme) -> Element<'a, Event, Theme, Renderer> {
        text("Queue Pane").into()
    }
}
