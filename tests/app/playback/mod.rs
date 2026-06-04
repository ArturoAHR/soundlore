use std::{thread, time::Duration};

use nameless_music_player_lib::playback::pipeline::event_emitter::AudioPipelineEvent;

use crate::{
    assert_timeout,
    common::{file::AUDIO_FILE_FIXTURES_PATH, playback::TestPlayback},
};

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

    let mut playback_engine = playback.playback_engine.borrow_mut();

    let sample_buffer_consumer = playback_engine.sample_buffer_consumer.as_mut().unwrap();

    assert_timeout!(
        !sample_buffer_consumer.is_empty(),
        Duration::from_millis(100),
        "Should have produced decoded samples and pushed them into the buffer"
    );
}

#[test]
fn decodes_different_sample_rates_and_channels() {
    let output_sample_rate = 44100;
    let output_channels = 2;

    let mut playback = TestPlayback::build(output_sample_rate, output_channels);

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    let sample_rates = vec![48000, 44100];
    let channel_counts = vec![1, 2];
    let formats = vec!["wav", "mp3", "ogg", "aac", "flac", "m4a", "aiff"];

    for sample_rate in sample_rates.iter() {
        for channels in channel_counts.iter() {
            for format in formats.iter() {
                let mut sample_count = 0;

                let channel_count_name = match channels {
                    1 => "mono",
                    2 => "stereo",
                    _ => unimplemented!(),
                };

                let file_name = format!("{}_{}.{}", sample_rate, channel_count_name, format);

                println!("{file_name}");

                playback
                    .playback_controller
                    .play(Some(
                        audio_file_fixtures_path
                            .all_sample_rates_and_channels
                            .join(&file_name),
                    ))
                    .unwrap();

                let mut playback_engine = playback.playback_engine.borrow_mut();

                loop {
                    let sample_buffer_consumer =
                        playback_engine.sample_buffer_consumer.as_mut().unwrap();

                    if sample_buffer_consumer.is_empty() {
                        if let Ok(event) = playback.playback_controller.poll_audio_pipeline_event()
                        {
                            match event {
                                Some(AudioPipelineEvent::EndOfTrack) => break,
                                _ => {
                                    thread::sleep(Duration::from_millis(100));
                                    continue;
                                }
                            }
                        }
                    }

                    let samples_contained = sample_buffer_consumer.slots();

                    let mut output_samples = vec![0.0; samples_contained];

                    sample_count += samples_contained;

                    let _ = sample_buffer_consumer.pop_entire_slice(&mut output_samples);
                }

                println!("Sample count for {file_name}: {sample_count}");

                // assert_eq!(
                //     sample_count,
                //     output_channels as usize * output_sample_rate as usize,
                //     "Sample count mismatch for file: {file_name}"
                // )
            }
        }
    }
}
