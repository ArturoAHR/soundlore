use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    message::Message,
    ui::{components::explorer_pane::Event, theme::Theme},
};

impl App {
    pub fn view_explorer_pane(&self) -> Element<'_, Message<app::Event>, Theme, Renderer> {
        self.explorer_pane
            .view(&self.theme)
            .map(Message::wrap_payload(app::Event::ExplorerPane))
    }

    pub fn handle_explorer_pane(&mut self, event: Message<Event>) -> Task<Message<app::Event>> {
        let (task, outcomes) = self.explorer_pane.update(event);
        let component_task = task.map(Message::wrap_payload(app::Event::ExplorerPane));

        if outcomes.len() == 0 {
            return component_task;
        };

        let mut tasks = vec![component_task];

        for outcome in outcomes {
            let outcome = match outcome.payload {};

            let outcome_task = self.handle_outcome(outcome);

            tasks.push(outcome_task);
        }

        Task::batch(tasks)
    }
}
