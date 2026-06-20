use std::{cell::RefCell, rc::Rc, sync::atomic::Ordering};

use nameless_music_player_lib::playback::PlaybackController;

use crate::common::{
    log::initialize_logging,
    playback::engine::{TestEngine, TestEngineContainer},
};

pub mod engine;

pub struct TestPlayback {
    pub playback_controller: PlaybackController,
    pub playback_engine: Rc<RefCell<TestEngine>>,
}

impl TestPlayback {
    pub fn build(output_sample_rate: u32, output_channels: u16) -> Self {
        initialize_logging();

        let playback_engine = Rc::new(RefCell::new(TestEngine::new(
            output_sample_rate,
            output_channels,
        )));

        let mut playback_controller =
            PlaybackController::new(Box::new(TestEngineContainer::new(playback_engine.clone())));

        playback_controller.initialize_output().unwrap();

        Self {
            playback_controller,
            playback_engine,
        }
    }

    pub fn consume_samples_buffer(&mut self) -> Vec<f32> {
        let mut playback_engine = self.playback_engine.borrow_mut();

        playback_engine.consume()
    }

    pub fn is_sample_buffer_empty(&self) -> bool {
        let playback_engine = self.playback_engine.borrow();

        let sample_buffer_consumer = playback_engine.sample_buffer_consumer.as_ref().unwrap();

        sample_buffer_consumer.is_empty()
    }

    pub fn sample_count(&self) -> usize {
        let playback_engine = self.playback_engine.borrow();

        playback_engine.samples_played.load(Ordering::Acquire) as usize
    }
}
