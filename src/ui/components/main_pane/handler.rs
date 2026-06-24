use iced::Task;

use crate::{
    app::{self, App, Message},
    ui::components::main_pane::{Event, Outcome},
};

impl App {
    pub fn handle_main_pane(&mut self, event: Event) -> Task<Message> {
        let (task, outcomes) = self.main_pane.update(event);
        let component_task = task.map(Message::MainPane);

        if outcomes.len() == 0 {
            return component_task;
        };

        let mut tasks = vec![component_task];

        for outcome in outcomes {
            let outcome = match outcome {
                Outcome::Playback(playback_outcome) => app::Outcome::Playback(playback_outcome),
            };

            let outcome_task = self.handle_outcome(outcome);

            tasks.push(outcome_task);
        }

        Task::batch(tasks)
    }
}
