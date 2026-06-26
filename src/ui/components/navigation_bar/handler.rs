use iced::{Element, Renderer, Task};
use rfd::AsyncFileDialog;

use crate::{
    app::{self, App},
    ui::{
        components::navigation_bar::{Message, Outcome},
        theme::Theme,
    },
};

impl App {
    pub fn view_navigation_bar(&self) -> Element<'_, app::Message, Theme, Renderer> {
        self.navigation_bar
            .view(&self.theme)
            .map(app::Message::NavigationBar)
    }

    pub fn handle_navigation_bar(&mut self, event: Message) -> Task<app::Message> {
        let (task, outcomes) = self.navigation_bar.update(event);
        let component_task = task.map(app::Message::NavigationBar);

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
                    app::Message::ScanDirectory,
                ),
            };
            tasks.push(outcome_task);
        }

        Task::batch(tasks)
    }
}
