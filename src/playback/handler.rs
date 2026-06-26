use iced::Task;
use tracing::error;

use crate::{
    app::{self, App},
    playback::{PlaybackControllerError, PlaybackControllerStatus, event::PlaybackControllerEvent},
    ui::components::playback_bar,
};

#[derive(Debug, Clone)]
pub enum Message {
    PendingOutputDeviceChange,
    OutputDeviceChanged,
    OutputDeviceChangeFailed(PlaybackControllerError),
    PollPlaybackEvent,
}

impl App {
    pub fn handle_playback(&mut self, message: Message) -> Task<app::Message> {
        match message {
            Message::PollPlaybackEvent => {
                while let Ok(Some(event)) = self.playback_controller.poll_event() {
                    match event {
                        PlaybackControllerEvent::EndOfTrack => {
                            // TODO: Implement playing next track.
                        }
                        _ => {}
                    }
                }

                if self.playback_controller.status == PlaybackControllerStatus::Stopped {
                    return Task::none();
                }

                let Some(track) = self.current_playing_track.as_ref() else {
                    return Task::none();
                };

                let Some(output_format) = self.playback_controller.output_format.as_ref() else {
                    return Task::none();
                };

                let audio_engine_generation =
                    self.playback_controller.get_audio_engine_generation();

                if audio_engine_generation <= self.playback_bar.seek_generation_threshold {
                    return Task::none();
                }

                let current_position = self.playback_controller.get_current_track_samples_played()
                    as f64
                    * (track.sample_rate as f64 / output_format.sample_rate as f64)
                    / output_format.channels as f64;

                Task::done(app::Message::PlaybackBar(
                    playback_bar::Message::PlaybackProgressed(current_position),
                ))
            }
            Message::PendingOutputDeviceChange => {
                if let Err(error) = self.playback_controller.initialize_output() {
                    return Task::done(app::Message::Playback(Message::OutputDeviceChangeFailed(
                        error,
                    )));
                }

                Task::done(app::Message::Playback(Message::OutputDeviceChanged))
            }
            Message::OutputDeviceChangeFailed(error) => {
                error!("Failed to initialize playback output: {error}");

                // TODO: Display error popup with user friendly message.

                Task::none()
            }
            _ => Task::none(),
        }
    }
}
