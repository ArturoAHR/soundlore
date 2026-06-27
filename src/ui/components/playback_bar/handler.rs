use iced::{Element, Renderer, Task};

use crate::{
    app::{self, App},
    event::Event,
    ui::{
        components::playback_bar::{
            Message, Outcome, PlaybackBarUpdateContext, PlaybackBarViewContext,
        },
        theme::Theme,
    },
};

impl App {
    pub fn view_playback_bar(&self) -> Element<'_, app::Message, Theme, Renderer> {
        let context = PlaybackBarViewContext {
            current_playing_track: &self.current_playing_track,
            theme: &self.theme,
        };

        self.playback_bar
            .view(context)
            .map(app::Message::PlaybackBar)
    }

    pub fn handle_playback_bar(&mut self, message: Message) -> Task<app::Message> {
        let playback_bar_context = PlaybackBarUpdateContext {
            playback_controller_status: &self.playback_controller.status,
            playback_engine_generation: self.playback_controller.get_audio_engine_generation(),
        };

        let (task, outcomes) = self.playback_bar.update(message, playback_bar_context);
        let component_task = task.map(app::Message::PlaybackBar);

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

    pub fn notify_playback_bar(&mut self, event: &Event) -> Task<app::Message> {
        self.playback_bar
            .on_event(event)
            .map(app::Message::PlaybackBar)
    }
}
