use iced::Task;
use rfd::AsyncFileDialog;

use crate::{
    app::{App, Message},
    ui::components::navigation_bar::{Event, Outcome},
};

impl App {
    pub fn handle_navigation_bar(&mut self, event: Event) -> Task<Message> {
        let (task, outcomes) = self.navigation_bar.update(event);
        let component_task = task.map(Message::NavigationBar);

        if outcomes.len() == 0 {
            return component_task;
        };

        let mut tasks = vec![component_task];
        for outcome in outcomes {
            let outcome_task = match outcome {
                Outcome::OpenSelectDirectoryDialog => Task::perform(
                    async {
                        AsyncFileDialog::new().pick_folders().await.map(|handles| {
                            handles.iter().map(|handle| handle.path().into()).collect()
                        })
                    },
                    Message::ScanDirectory,
                ),
            };
            tasks.push(outcome_task);
        }

        Task::batch(tasks)
    }
}
