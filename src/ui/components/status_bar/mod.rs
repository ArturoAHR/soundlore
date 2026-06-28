use iced::{
    Element, Length, Renderer, Task,
    widget::{container, text},
};
use tracing::instrument;

use crate::{app::AppStatus, event::Event, ui::theme::Theme};

pub mod handler;

#[derive(Debug)]
pub struct StatusBar {}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub enum Outcome {}

#[derive(Debug)]
pub struct StatusBarViewContext<'a> {
    pub theme: &'a Theme,
    pub status: &'a AppStatus,
}

#[derive(Debug)]
pub struct StatusBarUpdateContext {}

impl StatusBar {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Message) -> (Task<Message>, Vec<Outcome>) {
        (Task::none(), vec![])
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, _event: &Event) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(&'a self, ctx: StatusBarViewContext) -> Element<'a, Message, Theme, Renderer> {
        let status_label = match ctx.status {
            AppStatus::Idle => "",
            AppStatus::AddingTracks => "Adding tracks",
            AppStatus::FinishedAddingTracks => "Finished adding tracks",
        };

        container(text(status_label))
            .height(Length::Fixed(20.0))
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.palette.surface_sunken.into()),
                ..container::Style::default()
            })
            .into()
    }
}
