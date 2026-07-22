#![allow(clippy::future_not_send)]

use std::{cell::RefCell, rc::Rc};

use soundlore_lib::{
    app::{App, Message},
    playback::{PlaybackController, pipeline::thread::AudioPipelineThreadEvent},
    ui::theme::Theme,
};
use sqlx::SqlitePool;

use crate::common::{
    database::get_database_pool,
    emulation::perform_task,
    log::initialize_logging,
    playback::engine::{TestEngine, TestEngineContainer},
};

pub struct TestApp {
    pub app: App,
    pub pool: SqlitePool,
    #[allow(dead_code)]
    pub audio_pipeline_event_receiver:
        iced::futures::channel::mpsc::UnboundedReceiver<AudioPipelineThreadEvent>,
}

impl TestApp {
    pub async fn build() -> Self {
        initialize_logging();

        let pool = get_database_pool().await;

        let playback_engine = Rc::new(RefCell::new(TestEngine::new(44100, 2)));

        let mut playback_controller =
            PlaybackController::new(Box::new(TestEngineContainer::new(playback_engine.clone())));

        let (audio_pipeline_event_sender, audio_pipeline_event_receiver) =
            iced::futures::channel::mpsc::unbounded();

        playback_controller
            .initialize_playback(audio_pipeline_event_sender)
            .unwrap();

        let (mut app, initial_task) = App::new(pool.clone(), Theme::DARK, 1.0, playback_controller);

        perform_task(&mut app, initial_task).await;

        Self {
            app,
            pool,
            audio_pipeline_event_receiver,
        }
    }

    pub async fn dispatch_message(&mut self, message: Message) {
        let task = self.app.update(message);

        perform_task(&mut self.app, task).await;
    }

    pub fn state(&self) -> &App {
        &self.app
    }
}
