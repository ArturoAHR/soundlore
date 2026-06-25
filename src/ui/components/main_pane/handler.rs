use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App, Message},
    ui::{
        components::main_pane::{Event, MainPaneViewContext, Outcome},
        theme::Theme,
    },
};

impl App {
    pub fn view_main_pane(&self) -> Element<'_, Message, Theme, Renderer> {
        let context = MainPaneViewContext {
            theme: &self.theme,
            tracks: &self.tracks,
        };

        self.main_pane.view(context).map(Message::MainPane)
    }

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
