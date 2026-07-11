use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    event::Event,
    ui::{
        components::status_bar::{Message, StatusBarViewContext},
        theme::Theme,
    },
};

impl App {
    pub fn view_status_bar(&self) -> Element<'_, app::Message, Theme, Renderer> {
        let context = StatusBarViewContext {
            status: &self.status,
            theme: &self.theme,
        };

        self.status_bar.view(context).map(app::Message::StatusBar)
    }

    pub fn handle_status_bar(&mut self, event: Message) -> Task<app::Message> {
        let (task, _outcomes) = self.status_bar.update(event);
        let component_task = task.map(app::Message::StatusBar);

        // if outcomes.len() == 0 {
        component_task
        // };

        // let mut tasks = vec![component_task];

        // for outcome in outcomes {
        //     let outcome = match outcome {};

        //     let outcome_task = self.handle_outcome(outcome);

        //     tasks.push(outcome_task);
        // }

        // Task::batch(tasks)
    }

    pub fn notify_status_bar(&mut self, event: &Event) -> Task<app::Message> {
        self.status_bar.on_event(event).map(app::Message::StatusBar)
    }
}
