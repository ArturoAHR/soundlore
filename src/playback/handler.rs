use iced::Task;
use tracing::{error, instrument};

use crate::{
    app::{self, App},
    error::AppError,
    event::Event::AttemptedPlayingTrack,
    playback::{
        PlaybackControllerError, PlaybackControllerStatus,
        pipeline::thread::AudioPipelineThreadEvent,
    },
    track::models::TrackId,
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

                #[allow(clippy::single_match)]
                match event {
                    AudioPipelineThreadEvent::TrackFinished => {
                        let Some(next_track_id) =
                            self.current_playing_track_id.and_then(|track_id| {
                                self.displayed_track_ids
                                    .iter()
                                    .position(|&displayed_track_id| displayed_track_id == track_id)
                                    .and_then(|displayed_track_index| {
                                        self.displayed_track_ids.get(displayed_track_index + 1)
                                    })
                            })
                        else {
                            return Task::none();
                        };

                        match self.play_track(*next_track_id) {
                            Ok(event_tasks) => return event_tasks,
                            Err(error) => {
                                error!("Could not play next track: {error}");
                            }
                        }
                    }
                    _ => {}
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

    pub fn play_track(&mut self, track_id: TrackId) -> Result<Task<app::Message>, AppError> {
        let track = self
            .tracks
            .get(&track_id)
            .ok_or_else(|| AppError::TrackNotFound {
                id: Some(track_id),
                path: None,
            })?
            .clone();

        let event_tasks = self.broadcast(AttemptedPlayingTrack);

        self.playback_controller.play(track)?;

        self.current_playing_track_id = Some(track_id);

        Ok(event_tasks)
    }
}
