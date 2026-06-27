use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    event::Event,
    ui::{components::queue_pane::Message, theme::Theme},
};

impl App {
    pub fn view_queue_pane(&self) -> Element<'_, app::Message, Theme, Renderer> {
        self.queue_pane
            .view(&self.theme)
            .map(app::Message::QueuePane)
    }

    pub fn handle_queue_pane(&mut self, event: Message) -> Task<app::Message> {
        let (task, outcomes) = self.queue_pane.update(event);
        let component_task = task.map(app::Message::QueuePane);

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

    pub fn notify_queue_pane(&mut self, event: &Event) -> Task<app::Message> {
        self.queue_pane.on_event(event).map(app::Message::QueuePane)
    }
}
