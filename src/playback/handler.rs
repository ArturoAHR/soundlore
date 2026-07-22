use iced::Task;
use tracing::{error, instrument};

use crate::{
    app::{self, App},
    playback::{
        PlaybackControllerError, PlaybackControllerStatus,
        pipeline::thread::AudioPipelineThreadEvent,
    },
    ui::components::playback_bar,
};

#[derive(Debug, Clone)]
pub enum Message {
    AudioPipelineEvent(AudioPipelineThreadEvent),
    PendingOutputDeviceChange,
    OutputDeviceChanged,
    OutputDeviceChangeFailed(PlaybackControllerError),
    PollPlaybackCurrentPlaybackPosition,
}

impl App {
    #[instrument(skip(self))]
    pub fn handle_playback(&mut self, message: Message) -> Task<app::Message> {
        match message {
            Message::AudioPipelineEvent(event) => {
                if let Err(error) = self.playback_controller.handle_audio_pipeline_event(&event) {
                    error!("Playback controller failed to handle audio pipeline event: {error}");
                }

                Task::none()
            }
            Message::PollPlaybackCurrentPlaybackPosition => {
                if matches!(
                    self.playback_controller.status,
                    PlaybackControllerStatus::Stopped
                ) {
                    return Task::none();
                }

                let Some(track) = self
                    .current_playing_track_id
                    .and_then(|track_id| self.tracks.get(&track_id))
                else {
                    return Task::none();
                };

                let Some(output_format) = self.playback_controller.output_format.as_ref() else {
                    return Task::none();
                };

                let audio_engine_generation =
                    self.playback_controller.get_audio_engine_generation();

                if audio_engine_generation
                    <= self.playback_bar.current_position_generation_threshold
                {
                    return Task::none();
                }

                let current_position = self.playback_controller.get_current_track_samples_played()
                    as f64
                    * (track.sample_rate as f64 / f64::from(output_format.sample_rate))
                    / f64::from(output_format.channels);

                Task::done(app::Message::PlaybackBar(
                    playback_bar::Message::PlaybackProgressed(current_position),
                ))
            }
            Message::PendingOutputDeviceChange => {
                if let Err(error) = self.playback_controller.build_output() {
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
            Message::OutputDeviceChanged => Task::none(),
        }
    }
}
