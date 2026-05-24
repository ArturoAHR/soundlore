use iced::{widget::row, Element, Renderer, Task};
use tracing::instrument;

use crate::ui::{
    components::navigation_bar::navigation_bar_menu::navigation_bar_menu, theme::Theme,
};

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
    pub fn update(&mut self, event: Event) -> (Task<Event>, Option<Outcome>) {
        match event {
            Event::SelectedScanDirectoryOption => {
                (Task::none(), Some(Outcome::OpenSelectDirectoryDialog))
            }
        }
    }

    pub fn view<'a>(&'a self, theme: &Theme) -> Element<'a, Event, Theme, Renderer> {
        row![navigation_bar_menu(theme)].into()
    }
}
