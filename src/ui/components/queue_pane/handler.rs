use iced::Task;

use crate::{
    app::{App, Message},
    ui::components::queue_pane::Event,
};

impl App {
    pub fn handle_queue_pane(&mut self, event: Event) -> Task<Message> {
        let (task, outcomes) = self.queue_pane.update(event);
        let component_task = task.map(Message::QueuePane);

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
