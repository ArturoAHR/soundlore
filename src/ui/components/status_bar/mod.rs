use iced::{Element, Renderer, Task, widget::text};
use tracing::instrument;

use crate::{app::AppStatus, ui::theme::Theme};

pub mod handler;

#[derive(Debug)]
pub struct StatusBar {}

#[derive(Debug, Clone)]
pub enum Event {}

#[derive(Debug, Clone)]
pub enum Outcome {}

impl StatusBar {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Event) -> (Task<Event>, Vec<Outcome>) {
        (Task::none(), vec![])
    }

    pub fn view<'a>(
        &'a self,
        _theme: &Theme,
        status: &AppStatus,
    ) -> Element<'a, Event, Theme, Renderer> {
        let status_label = match status {
            AppStatus::Idle => "",
            AppStatus::AddingTracks => "Adding tracks",
            AppStatus::FinishedAddingTracks => "Finished adding tracks",
        };

        text(status_label).into()
    }
}
