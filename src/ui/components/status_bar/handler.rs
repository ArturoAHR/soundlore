use iced::{Element, Renderer, Task};

use crate::{
    app::{App, Message},
    ui::{
        components::status_bar::{Event, StatusBarViewContext},
        theme::Theme,
    },
};

impl App {
    pub fn view_status_bar(&self) -> Element<'_, Message, Theme, Renderer> {
        let context = StatusBarViewContext {
            status: &self.status,
            theme: &self.theme,
        };

        self.status_bar.view(context).map(Message::StatusBar)
    }

    pub fn handle_status_bar(&mut self, event: Event) -> Task<Message> {
        let (task, outcomes) = self.status_bar.update(event);
        let component_task = task.map(Message::StatusBar);

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
