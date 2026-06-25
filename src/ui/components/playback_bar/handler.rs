use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    message::Message,
    ui::{
        components::playback_bar::{
            Event, Outcome, PlaybackBarUpdateContext, PlaybackBarViewContext,
        },
        theme::Theme,
    },
};

impl App {
    pub fn view_playback_bar(&self) -> Element<'_, Message<app::Event>, Theme, Renderer> {
        let context = PlaybackBarViewContext {
            current_playing_track: &self.current_playing_track,
            theme: &self.theme,
        };

        self.playback_bar
            .view(context)
            .map(Message::wrap_payload(app::Event::PlaybackBar))
    }

    pub fn handle_playback_bar(&mut self, event: Message<Event>) -> Task<Message<app::Event>> {
        let playback_bar_context = PlaybackBarUpdateContext {
            playback_controller_status: &self.playback_controller.status,
            playback_engine_generation: self.playback_controller.get_audio_engine_generation(),
        };

        let (task, outcomes) = self.playback_bar.update(event, playback_bar_context);
        let component_task = task.map(Message::wrap_payload(app::Event::PlaybackBar));

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
