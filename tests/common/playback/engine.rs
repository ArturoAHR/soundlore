use std::{cell::RefCell, rc::Rc};

use nameless_music_player_lib::playback::{
    engine::{PlaybackEngine, PlaybackEngineError},
    PlaybackController,
};
use rtrb::Consumer;

pub struct TestEngine {
    pub sample_buffer_consumer: Option<Consumer<f32>>,
    pub output_sample_rate: u32,
    pub output_channels: u16,
}

pub struct TestEngineContainer {
    pub engine: Rc<RefCell<TestEngine>>,
}

impl TestEngine {
    pub fn new(output_sample_rate: u32, output_channels: u16) -> Self {
        Self {
            output_sample_rate,
            output_channels,
            sample_buffer_consumer: None,
        }
    }
}

impl PlaybackEngine for TestEngine {
    fn build_stream(
        &mut self,
        sample_buffer_consumer: rtrb::Consumer<f32>,
    ) -> Result<(u32, u16), PlaybackEngineError> {
        self.sample_buffer_consumer = Some(sample_buffer_consumer);

        Ok((self.output_sample_rate, self.output_channels))
    }

    fn play_stream(&self) -> Result<(), PlaybackEngineError> {
        Ok(())
    }

    fn pause_stream(&self) -> Result<(), PlaybackEngineError> {
        Ok(())
    }
}

impl TestEngineContainer {
    pub fn new(engine: Rc<RefCell<TestEngine>>) -> Self {
        Self { engine }
    }
}

impl PlaybackEngine for TestEngineContainer {
    fn build_stream(
        &mut self,
        sample_buffer_consumer: Consumer<f32>,
    ) -> Result<(u32, u16), PlaybackEngineError> {
        self.engine
            .borrow_mut()
            .build_stream(sample_buffer_consumer)
    }

    fn pause_stream(&self) -> Result<(), PlaybackEngineError> {
        self.engine.borrow().pause_stream()
    }

    fn play_stream(&self) -> Result<(), PlaybackEngineError> {
        self.engine.borrow().play_stream()
    }
}
