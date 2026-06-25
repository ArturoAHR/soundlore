use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    message::Message,
    ui::{components::queue_pane::Event, theme::Theme},
};

impl App {
    pub fn view_queue_pane(&self) -> Element<'_, Message<app::Event>, Theme, Renderer> {
        self.queue_pane
            .view(&self.theme)
            .map(Message::wrap_payload(app::Event::QueuePane))
    }

    pub fn handle_queue_pane(&mut self, event: Message<Event>) -> Task<Message<app::Event>> {
        let (task, outcomes) = self.queue_pane.update(event);
        let component_task = task.map(Message::wrap_payload(app::Event::QueuePane));

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
