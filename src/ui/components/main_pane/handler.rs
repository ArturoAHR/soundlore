use iced::Task;

use crate::{
    app::{self, App, Message},
    ui::components::main_pane::{Event, Outcome},
};

impl App {
    pub fn handle_main_pane(&mut self, event: Event) -> Task<Message> {
        let (task, outcome) = self.main_pane.update(event);
        let component_task = task.map(Message::MainPane);

        let Some(outcome) = outcome else {
            return component_task;
        };

        let outcome = match outcome {
            Outcome::Playback(outcome) => app::Outcome::Playback(outcome),
        };

        let outcome_task = self.handle_outcome(outcome);

        Task::batch([outcome_task, component_task])
    }
}
