use iced::Task;

use crate::{
    app::{App, Message},
    ui::components::track_information_pane::Event,
};

impl App {
    pub fn handle_track_information_pane(&mut self, event: Event) -> Task<Message> {
        let (task, outcomes) = self.track_information_pane.update(event);
        let component_task = task.map(Message::TrackInformationPane);

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
