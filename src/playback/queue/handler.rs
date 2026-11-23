use iced::Task;
use tracing::{info, instrument, warn};

use crate::{
    app::{self, App},
    error::AppError,
    playback::queue::PlaybackRepeatMode,
};

#[derive(Debug, Clone)]
#[expect(clippy::empty_enums, reason = "placeholder")]
pub enum Message {
    // GenerateTrackPoolShuffleWeights
    // FinishedTrackPoolShuffleWeightGeneration(Result<Vec<?>, PlaybackQueueError>)
}

impl App {
    #[instrument(skip(self))]
    pub fn play_next_track(&mut self) -> Result<Task<app::Message>, AppError> {
        let next_track_id = if matches!(
            self.playback_queue.repeat_mode,
            PlaybackRepeatMode::RepeatOne
        ) {
            self.playback_queue.current()
        } else {
            self.playback_queue.next()
        };

        let task = if let Some(next_track_id) = next_track_id {
            self.play_track(next_track_id)?
        } else {
            info!("Found no more track in the queue");

            Task::none()
        };

        Ok(task)
    }
}
