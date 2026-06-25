use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    message::Message,
    ui::{
        components::status_bar::{Event, StatusBarViewContext},
        theme::Theme,
    },
};

impl App {
    pub fn view_status_bar(&self) -> Element<'_, Message<app::Event>, Theme, Renderer> {
        let context = StatusBarViewContext {
            status: &self.status,
            theme: &self.theme,
        };

        self.status_bar
            .view(context)
            .map(Message::wrap_payload(app::Event::StatusBar))
    }

    pub fn handle_status_bar(&mut self, event: Message<Event>) -> Task<Message<app::Event>> {
        let (task, outcomes) = self.status_bar.update(event);
        let component_task = task.map(Message::wrap_payload(app::Event::StatusBar));

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
