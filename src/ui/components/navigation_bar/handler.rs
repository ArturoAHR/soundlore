use iced::{Element, Renderer, Task};
use rfd::AsyncFileDialog;

use crate::{
    app::{self, App},
    message::Message,
    ui::{
        components::navigation_bar::{Event, Outcome},
        theme::Theme,
    },
};

impl App {
    pub fn view_navigation_bar(&self) -> Element<'_, Message<app::Event>, Theme, Renderer> {
        self.navigation_bar
            .view(&self.theme)
            .map(Message::wrap_payload(app::Event::NavigationBar))
    }

    pub fn handle_navigation_bar(&mut self, event: Message<Event>) -> Task<Message<app::Event>> {
        let (task, outcomes) = self.navigation_bar.update(event);
        let component_task = task.map(Message::wrap_payload(app::Event::NavigationBar));

        if outcomes.len() == 0 {
            return component_task;
        };

        let mut tasks = vec![component_task];
        for outcome in outcomes {
            let outcome_task = match outcome.payload {
                Outcome::OpenSelectDirectoryDialog => outcome.task_from(
                    async {
                        AsyncFileDialog::new().pick_folders().await.map(|handles| {
                            handles.iter().map(|handle| handle.path().into()).collect()
                        })
                    },
                    app::Event::ScanDirectory,
                ),
            };
            tasks.push(outcome_task);
        }

        Task::batch(tasks)
    }
}
