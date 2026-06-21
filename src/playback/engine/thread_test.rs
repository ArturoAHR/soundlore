use cpal::{OutputStreamTimestamp, StreamInstant};
use pretty_assertions::assert_eq;
use rtrb::{Producer, RingBuffer};

use crate::playback::constants::SAMPLE_BUFFER_CAPACITY;

use super::*;

const DEFAULT_OUTPUT_DATA_SIZE: usize = 1024;

struct TestAudioEngineDataProcessor {
    processor: AudioEngineDataProcessor,
    sample_buffer_producer: Producer<f32>,
}

impl TestAudioEngineDataProcessor {
    pub fn new() -> Self {
        let (sample_buffer_producer, sample_buffer_consumer) =
            RingBuffer::new(SAMPLE_BUFFER_CAPACITY);

        Self {
            processor: AudioEngineDataProcessor {
                sample_buffer_consumer,
                samples_played: Arc::new(AtomicU64::default()),
                track_start_timestamp: Arc::new(AtomicI64::default()),
                samples_played_timestamp_offset: Arc::new(AtomicU64::default()),
                generation_counter: Arc::new(GenerationCounter::default()),
                paused: Arc::new(AtomicBool::default()),
            },
            sample_buffer_producer,
        }
    }

    pub fn process<T>(&mut self, data: &mut [T])
    where
        T: SizedSample + FromSample<f32>,
    {
        self.processor.process(
            data,
            &OutputCallbackInfo::new(OutputStreamTimestamp {
                callback: StreamInstant::new(0, 0),
                playback: StreamInstant::new(0, 0),
            }),
        );
    }

    pub fn buffer(&mut self, data: &[f32]) {
        self.sample_buffer_producer.push_entire_slice(data).unwrap();
    }

    pub fn set_paused(&self, value: bool) {
        self.processor.paused.store(value, Ordering::Relaxed);
    }

    pub fn set_samples_played_timestamp_offset(&self, value: u64) {
        self.processor
            .samples_played_timestamp_offset
            .store(value, Ordering::Relaxed);
    }

    pub fn increase_audio_pipeline_generation_counter(&self) {
        self.processor
            .generation_counter
            .audio_pipeline
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn assert_samples_played(&self, expected_amount: u64) {
        assert_eq!(
            expected_amount,
            self.processor.samples_played.load(Ordering::Relaxed),
            "Samples played weren't properly updated"
        );
    }

    pub fn assert_engine_generation_counter(&self, expected_amount: u64) {
        assert_eq!(
            expected_amount,
            self.processor
                .generation_counter
                .audio_engine
                .load(Ordering::Relaxed),
            "Engine generation counter wasn't properly updated"
        );
    }

    pub fn assert_track_start_timestamp(&self, expected_amount: i64) {
        assert_eq!(
            expected_amount,
            self.processor.track_start_timestamp.load(Ordering::Relaxed),
            "Track start timestamp wasn't properly updated"
        );
    }

    pub fn assert_sample_buffer_consumer_slots(&self, expected_amount: usize) {
        assert_eq!(
            expected_amount,
            self.processor.sample_buffer_consumer.slots(),
            "Ring buffer contents aren't correct"
        );
    }
}

#[test]
fn should_consume_samples_from_buffer() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);

    let mut data = vec![0.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 1.0,
            "Output buffer wasn't filled correctly with sound."
        )
    });

    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_samples_played(DEFAULT_OUTPUT_DATA_SIZE as u64);
}

#[test]
fn should_consume_samples_from_partially_filled_buffer() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE / 2]);

    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().enumerate().for_each(|(index, sample)| {
        if index < DEFAULT_OUTPUT_DATA_SIZE / 2 {
            assert_eq!(
                *sample, 1.0,
                "Output buffer wasn't filled correctly with sound at position {index}."
            )
        } else {
            assert_eq!(
                *sample, 0.0,
                "Output buffer wasn't filled correctly with silence at position {index}."
            )
        }
    });

    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_samples_played(DEFAULT_OUTPUT_DATA_SIZE as u64 / 2);
}

#[test]
fn should_continuously_consume_samples_from_buffer() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE * 3]);

    let mut data = vec![0.0; DEFAULT_OUTPUT_DATA_SIZE];

    for _ in 0..3 {
        harness.process(&mut data);

        data.iter().for_each(|sample| {
            assert_eq!(
                *sample, 1.0,
                "Output buffer wasn't filled correctly with sound."
            )
        });
    }

    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_samples_played(3 * DEFAULT_OUTPUT_DATA_SIZE as u64);
    harness.assert_track_start_timestamp(0);
    harness.assert_engine_generation_counter(0);
}

#[test]
fn should_underrun_with_silence() {
    let mut harness = TestAudioEngineDataProcessor::new();

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_samples_played(0);
    harness.assert_track_start_timestamp(0);
    harness.assert_engine_generation_counter(0);
}

#[test]
fn should_output_silence_if_paused() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
    harness.set_paused(true);

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_sample_buffer_consumer_slots(DEFAULT_OUTPUT_DATA_SIZE);
    harness.assert_samples_played(0);
    harness.assert_track_start_timestamp(0);
    harness.assert_engine_generation_counter(0);
}

#[test]
fn should_consume_buffer_after_pause() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
    harness.set_paused(true);

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_sample_buffer_consumer_slots(DEFAULT_OUTPUT_DATA_SIZE);
    harness.assert_samples_played(0);
    harness.assert_track_start_timestamp(0);
    harness.assert_engine_generation_counter(0);

    harness.set_paused(false);

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 1.0,
            "Output buffer wasn't filled correctly with sound."
        )
    });

    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_samples_played(DEFAULT_OUTPUT_DATA_SIZE as u64);
    harness.assert_track_start_timestamp(0);
    harness.assert_engine_generation_counter(0);
}

#[test]
fn should_clear_buffer_if_pipeline_generation_counter_is_increased() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
    harness.increase_audio_pipeline_generation_counter();

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_engine_generation_counter(1);
    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_samples_played(0);
    harness.assert_track_start_timestamp(0);
}

#[test]
fn should_consume_buffer_after_generation_counter_increase_clear() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
    harness.increase_audio_pipeline_generation_counter();

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_samples_played(0);
    harness.assert_engine_generation_counter(1);
    harness.assert_track_start_timestamp(0);

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);

    data = vec![0.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 1.0,
            "Output buffer wasn't filled correctly with sound."
        )
    });

    harness.assert_samples_played(DEFAULT_OUTPUT_DATA_SIZE as u64);
    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_track_start_timestamp(0);
}

#[test]
fn should_clear_buffer_if_pipeline_generation_counter_is_increased_twice_before_processing() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
    harness.increase_audio_pipeline_generation_counter();
    harness.increase_audio_pipeline_generation_counter();

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_engine_generation_counter(2);
    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_track_start_timestamp(0);

    data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });
}

#[test]
fn should_clear_buffer_if_pipeline_generation_counter_is_increased_twice_in_a_row() {
    let mut harness = TestAudioEngineDataProcessor::new();

    for index in 0..2 {
        harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
        harness.increase_audio_pipeline_generation_counter();

        // -1.0 so we can determine if silence was output.
        let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

        harness.process(&mut data);

        data.iter().for_each(|sample| {
            assert_eq!(
                *sample, 0.0,
                "Output buffer wasn't filled correctly with silence."
            )
        });

        harness.assert_engine_generation_counter(index + 1);
        harness.assert_sample_buffer_consumer_slots(0);
        harness.assert_track_start_timestamp(0);
    }
}

#[test]
fn should_clear_buffer_if_pipeline_generation_counter_is_increased_while_paused() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
    harness.increase_audio_pipeline_generation_counter();
    harness.set_paused(true);

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);
    harness.assert_engine_generation_counter(1);
    harness.assert_sample_buffer_consumer_slots(0);
}

#[test]
fn should_set_track_started_timestamp_with_offset_after_clearing() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE * 2]);

    let mut data = vec![0.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 1.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_sample_buffer_consumer_slots(DEFAULT_OUTPUT_DATA_SIZE);
    harness.assert_samples_played(DEFAULT_OUTPUT_DATA_SIZE as u64);

    harness.increase_audio_pipeline_generation_counter();
    harness.set_samples_played_timestamp_offset(DEFAULT_OUTPUT_DATA_SIZE as u64 / 2);

    // -1.0 so we can determine if silence was output.
    data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_engine_generation_counter(1);
    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_track_start_timestamp(DEFAULT_OUTPUT_DATA_SIZE as i64 / 2);
}

#[test]
fn should_set_negative_track_started_timestamp_with_offset_after_clearing() {
    let mut harness = TestAudioEngineDataProcessor::new();

    harness.buffer(&vec![1.0; DEFAULT_OUTPUT_DATA_SIZE]);
    harness.increase_audio_pipeline_generation_counter();
    harness.set_samples_played_timestamp_offset(DEFAULT_OUTPUT_DATA_SIZE as u64 / 2);

    // -1.0 so we can determine if silence was output.
    let mut data = vec![-1.0; DEFAULT_OUTPUT_DATA_SIZE];

    harness.process(&mut data);

    data.iter().for_each(|sample| {
        assert_eq!(
            *sample, 0.0,
            "Output buffer wasn't filled correctly with silence."
        )
    });

    harness.assert_engine_generation_counter(1);
    harness.assert_sample_buffer_consumer_slots(0);
    harness.assert_track_start_timestamp(DEFAULT_OUTPUT_DATA_SIZE as i64 / -2);
}
