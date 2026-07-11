use iced::{
    Element, Length, Renderer, Task,
    widget::{container, text},
};
use tracing::instrument;

use crate::{event::Event, ui::theme::Theme};

pub mod handler;

#[derive(Debug)]
pub struct TrackInformationPane {}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub enum Outcome {}

impl TrackInformationPane {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Message) -> (Task<Message>, Vec<Outcome>) {
        (Task::none(), vec![])
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, event: &Event) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(&'a self, _theme: &Theme) -> Element<'a, Message, Theme, Renderer> {
        container(text("Track Information Pane"))
            .height(Length::FillPortion(3))
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.palette.surface_raised.into()),
                ..container::Style::default()
            })
            .into()
    }
}
