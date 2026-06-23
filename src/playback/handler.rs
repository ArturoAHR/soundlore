use iced::Task;
use tracing::error;

use crate::{
    app::{App, Message},
    playback::{PlaybackControllerError, PlaybackControllerStatus, event::PlaybackControllerEvent},
    ui::components::playback_bar,
};

#[derive(Debug, Clone)]
pub enum PlaybackMessage {
    PendingOutputDeviceChange,
    OutputDeviceChanged,
    OutputDeviceChangeFailed(PlaybackControllerError),
    PollPlaybackEvent,
}

impl App {
    pub fn handle_playback(&mut self, message: PlaybackMessage) -> Task<Message> {
        match message {
            PlaybackMessage::PollPlaybackEvent => {
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

                Task::done(Message::PlaybackBar(
                    playback_bar::Event::PlaybackProgressed(current_position),
                ))
            }
            PlaybackMessage::PendingOutputDeviceChange => {
                if let Err(error) = self.playback_controller.initialize_output() {
                    return Task::done(Message::Playback(
                        PlaybackMessage::OutputDeviceChangeFailed(error),
                    ));
                }

                Task::done(Message::Playback(PlaybackMessage::OutputDeviceChanged))
            }
            PlaybackMessage::OutputDeviceChangeFailed(error) => {
                error!("Failed to initialize playback output: {error}");

                // TODO: Display error popup with user friendly message.

                Task::none()
            }
            _ => Task::none(),
        }
    }
}
