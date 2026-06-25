use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    message::Message,
    ui::{components::track_information_pane::Event, theme::Theme},
};

impl App {
    pub fn view_track_information_pane(&self) -> Element<'_, Message<app::Event>, Theme, Renderer> {
        self.track_information_pane
            .view(&self.theme)
            .map(Message::wrap_payload(app::Event::TrackInformationPane))
    }

    pub fn handle_track_information_pane(
        &mut self,
        event: Message<Event>,
    ) -> Task<Message<app::Event>> {
        let (task, outcomes) = self.track_information_pane.update(event);
        let component_task = task.map(Message::wrap_payload(app::Event::TrackInformationPane));

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
}
