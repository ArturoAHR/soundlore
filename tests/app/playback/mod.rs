use std::{thread, time::Duration};

use crate::common::{file::AUDIO_FILE_FIXTURES_PATH, playback::TestPlayback};

#[test]
fn decodes_samples_into_consumer() {
    let mut playback = TestPlayback::build(48000, 2);

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    playback
        .playback_controller
        .play(Some(
            format!(
                "{}/track.mp3",
                audio_file_fixtures_path.all_formats.to_str().unwrap()
            )
            .into(),
        ))
        .unwrap();

    thread::sleep(Duration::from_millis(50));

    let mut playback_engine = playback.playback_engine.borrow_mut();

    let sample_buffer_consumer = playback_engine.sample_buffer_consumer.as_mut().unwrap();

    assert!(!sample_buffer_consumer.is_empty())
}
