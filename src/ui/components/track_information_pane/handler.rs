use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    event::Event,
    ui::{components::track_information_pane::Message, theme::Theme},
};

impl App {
    pub fn view_track_information_pane(&self) -> Element<'_, app::Message, Theme, Renderer> {
        self.track_information_pane
            .view(&self.theme)
            .map(app::Message::TrackInformationPane)
    }

    pub fn handle_track_information_pane(&mut self, event: Message) -> Task<app::Message> {
        let (task, outcomes) = self.track_information_pane.update(event);
        let component_task = task.map(app::Message::TrackInformationPane);

        if outcomes.len() == 0 {
            return component_task;
        };

        let mut tasks = vec![component_task];

        for outcome in outcomes {
            let outcome = match outcome {};

            let outcome_task = self.handle_outcome(outcome);

            tasks.push(outcome_task);
        }

        Task::batch(tasks)
    }

    pub fn notify_track_information_pane(&mut self, event: &Event) -> Task<app::Message> {
        self.track_information_pane
            .on_event(event)
            .map(app::Message::TrackInformationPane)
    }
}
