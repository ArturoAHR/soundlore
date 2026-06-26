use iced::{Element, Renderer, Task, widget::text};
use tracing::instrument;

use crate::{app::AppStatus, ui::theme::Theme};

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

    pub fn view<'a>(&'a self, ctx: StatusBarViewContext) -> Element<'a, Message, Theme, Renderer> {
        let status_label = match ctx.status {
            AppStatus::Idle => "",
            AppStatus::AddingTracks => "Adding tracks",
            AppStatus::FinishedAddingTracks => "Finished adding tracks",
        };

        text(status_label).into()
    }
}
