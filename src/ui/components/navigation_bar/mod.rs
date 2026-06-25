use iced::{Element, Renderer, Task, widget::row};
use tracing::instrument;

use crate::{
    message::Message,
    ui::{components::navigation_bar::navigation_bar_menu::navigation_bar_menu, theme::Theme},
};

pub mod handler;
pub mod navigation_bar_menu;

#[derive(Debug)]
pub struct NavigationBar {}

#[derive(Debug, Clone)]
pub enum Event {
    SelectedScanDirectoryOption,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    OpenSelectDirectoryDialog,
}

impl NavigationBar {
    #[instrument(skip(self), level = "debug")]
    pub fn update(
        &mut self,
        event: Message<Event>,
    ) -> (Task<Message<Event>>, Vec<Message<Outcome>>) {
        match event.payload {
            Event::SelectedScanDirectoryOption => (
                Task::none(),
                vec![event.new_from(Outcome::OpenSelectDirectoryDialog)],
            ),
        }
    }

    pub fn view<'a>(&'a self, theme: &Theme) -> Element<'a, Message<Event>, Theme, Renderer> {
        row![navigation_bar_menu(theme)].into()
    }
}
