use iced::Task;

use crate::{
    app::{App, Message},
    track::models::Track,
};

#[derive(Debug, Clone)]
pub enum Event {
    AttemptedPlayingTrack,
    ActiveTrackChanged(Option<Track>),
    StartedPlayback,
    StoppedPlayback,
    EndOfTrack,
}

impl App {
    pub fn broadcast(&mut self, event: Event) -> Task<Message> {
        Task::batch(vec![
            self.notify_explorer_pane(&event),
            self.notify_main_pane(&event),
            self.notify_navigation_bar(&event),
            self.notify_playback_bar(&event),
            self.notify_queue_pane(&event),
            self.notify_status_bar(&event),
            self.notify_track_information_pane(&event),
        ])
    }
}
