use iced::{Element, Renderer, Task, widget::text};
use tracing::instrument;

use crate::{message::Message, ui::theme::Theme};

pub mod handler;

#[derive(Debug)]
pub struct TrackInformationPane {}

#[derive(Debug, Clone)]
pub enum Event {}

#[derive(Debug, Clone)]
pub enum Outcome {}

impl TrackInformationPane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(
        &mut self,
        event: Message<Event>,
    ) -> (Task<Message<Event>>, Vec<Message<Outcome>>) {
        (Task::none(), vec![])
    }

    pub fn view<'a>(&'a self, _theme: &Theme) -> Element<'a, Message<Event>, Theme, Renderer> {
        text("Track Information Pane").into()
    }
}
