use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    message::Message,
    ui::{
        components::main_pane::{Event, MainPaneViewContext, Outcome},
        theme::Theme,
    },
};

impl App {
    pub fn view_main_pane(&self) -> Element<'_, Message<app::Event>, Theme, Renderer> {
        let context = MainPaneViewContext {
            theme: &self.theme,
            tracks: &self.tracks,
        };

        self.main_pane
            .view(context)
            .map(Message::wrap_payload(app::Event::MainPane))
    }

    pub fn handle_main_pane(&mut self, event: Message<Event>) -> Task<Message<app::Event>> {
        let (task, outcomes) = self.main_pane.update(event);
        let component_task = task.map(Message::wrap_payload(app::Event::MainPane));

        if outcomes.len() == 0 {
            return component_task;
        };

        let mut tasks = vec![component_task];

        for outcome in outcomes {
            let outcome = match outcome.payload {
                Outcome::Playback(ref playback_outcome) => {
                    outcome.new_from(app::Outcome::Playback(playback_outcome.clone()))
                }
            };

            let outcome_task = self.handle_outcome(outcome);

            tasks.push(outcome_task);
        }

        Task::batch(tasks)
    }
}
