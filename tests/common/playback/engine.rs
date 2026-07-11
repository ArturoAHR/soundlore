use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

use rtrb::Consumer;
use soundlore_lib::playback::{
    GenerationCounter,
    engine::{PlaybackEngine, PlaybackEngineError, PlaybackEngineStatus},
};

pub struct TestEngine {
    pub sample_buffer_consumer: Option<Consumer<f32>>,
    pub output_sample_rate: u32,
    pub output_channels: u16,

    pub samples_played: Arc<AtomicU64>,
    pub track_start_timestamp: Arc<AtomicI64>,
    pub samples_played_timestamp_offset: Arc<AtomicU64>,
    pub generation_counter: Arc<GenerationCounter>,
    pub status: PlaybackEngineStatus,
}

pub struct TestEngineContainer {
    pub status: PlaybackEngineStatus,
    pub engine: Rc<RefCell<TestEngine>>,
}

impl TestEngine {
    pub fn new(output_sample_rate: u32, output_channels: u16) -> Self {
        Self {
            output_sample_rate,
            output_channels,
            sample_buffer_consumer: None,
            status: PlaybackEngineStatus::Playing,

            samples_played: Arc::new(AtomicU64::new(0)),
            samples_played_timestamp_offset: Arc::new(AtomicU64::new(0)),
            track_start_timestamp: Arc::new(AtomicI64::new(0)),
            generation_counter: Arc::new(GenerationCounter {
                audio_pipeline: AtomicU64::new(0),
                audio_engine: AtomicU64::new(0),
            }),
        }
    }

    pub fn consume(&mut self) -> Vec<f32> {
        let audio_pipeline_generation = self
            .generation_counter
            .audio_pipeline
            .load(Ordering::Acquire);
        let audio_engine_generation = self.generation_counter.audio_engine.load(Ordering::Relaxed);

        if audio_engine_generation != audio_pipeline_generation {
            let timestamp = self.samples_played.load(Ordering::Relaxed) as i64
                - self.samples_played_timestamp_offset.load(Ordering::Relaxed) as i64;

            self.track_start_timestamp
                .store(timestamp, Ordering::Relaxed);

            let output_samples = self.drain_sample_buffer();

            self.generation_counter
                .audio_engine
                .store(audio_pipeline_generation, Ordering::Release);

            return output_samples;
        }

        let output_samples = self.drain_sample_buffer();

        self.samples_played
            .fetch_add(output_samples.len() as u64, Ordering::Release);

        output_samples
    }

    pub fn drain_sample_buffer(&mut self) -> Vec<f32> {
        let sample_buffer_consumer = self.sample_buffer_consumer.as_mut().unwrap();

        let samples_contained = sample_buffer_consumer.slots();

        let mut output_samples = vec![0.0; samples_contained];

        let _ = sample_buffer_consumer.pop_entire_slice(&mut output_samples);

        output_samples
    }
}

impl PlaybackEngine for TestEngine {
    fn build_stream(
        &mut self,
        sample_buffer_consumer: rtrb::Consumer<f32>,
        samples_played: Arc<AtomicU64>,
        track_start_timestamp: Arc<AtomicI64>,
        samples_played_timestamp_offset: Arc<AtomicU64>,
        generation_counter: Arc<GenerationCounter>,
    ) -> Result<(u32, u16), PlaybackEngineError> {
        self.sample_buffer_consumer = Some(sample_buffer_consumer);
        self.samples_played = samples_played;
        self.track_start_timestamp = track_start_timestamp;
        self.samples_played_timestamp_offset = samples_played_timestamp_offset;
        self.generation_counter = generation_counter;

        Ok((self.output_sample_rate, self.output_channels))
    }

    fn pause(&mut self) -> Result<(), PlaybackEngineError> {
        self.status = PlaybackEngineStatus::Playing;

        Ok(())
    }

    fn play(&mut self) -> Result<(), PlaybackEngineError> {
        self.status = PlaybackEngineStatus::Paused;

        Ok(())
    }

    fn status(&self) -> &PlaybackEngineStatus {
        &self.status
    }
}

impl TestEngineContainer {
    pub fn new(engine: Rc<RefCell<TestEngine>>) -> Self {
        Self {
            engine,
            status: PlaybackEngineStatus::Playing,
        }
    }
}

impl PlaybackEngine for TestEngineContainer {
    fn build_stream(
        &mut self,
        sample_buffer_consumer: Consumer<f32>,
        samples_played: Arc<AtomicU64>,
        track_start_timestamp: Arc<AtomicI64>,
        samples_played_timestamp_offset: Arc<AtomicU64>,
        generation_counter: Arc<GenerationCounter>,
    ) -> Result<(u32, u16), PlaybackEngineError> {
        self.engine.borrow_mut().build_stream(
            sample_buffer_consumer,
            samples_played,
            track_start_timestamp,
            samples_played_timestamp_offset,
            generation_counter,
        )
    }

    fn play(&mut self) -> Result<(), PlaybackEngineError> {
        let result = self.engine.borrow_mut().play();

        self.status.clone_from(self.engine.borrow().status());

        result
    }

    fn pause(&mut self) -> Result<(), PlaybackEngineError> {
        let result = self.engine.borrow_mut().pause();

        self.status.clone_from(self.engine.borrow().status());

        result
    }

    fn status(&self) -> &PlaybackEngineStatus {
        &self.status
    }
}
