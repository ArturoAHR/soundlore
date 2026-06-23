use iced::Task;

use crate::{
    app::{self, App, Message},
    ui::components::playback_bar::{Event, Outcome, PlaybackBarUpdateContext},
};

impl App {
    pub fn handle_playback_bar(&mut self, event: Event) -> Task<Message> {
        let playback_bar_context = PlaybackBarUpdateContext {
            playback_controller_status: &self.playback_controller.status,
            playback_engine_generation: self.playback_controller.get_audio_engine_generation(),
        };

        let (task, outcome) = self.playback_bar.update(event, playback_bar_context);
        let component_task = task.map(Message::PlaybackBar);

        let Some(outcome) = outcome else {
            return component_task;
        };

        let outcome = match outcome {
            Outcome::Playback(playback_outcome) => app::Outcome::Playback(playback_outcome),
        };

        let outcome_task = self.handle_outcome(outcome);

        Task::batch([outcome_task, component_task])
    }
}
