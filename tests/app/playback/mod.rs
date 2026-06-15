use std::{
    thread::{self},
    time::Duration,
};

use nameless_music_player_lib::playback::pipeline::thread::AudioPipelineThreadEvent;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    common::{
        constants::{SUPPORTED_CHANNEL_COUNTS, SUPPORTED_FILE_FORMATS, SUPPORTED_SAMPLE_RATES},
        file::AUDIO_FILE_FIXTURES_PATH,
        playback::TestPlayback,
        track::create_mock_track,
    },
    playback::{assert::assert_sample_count, utils::get_file_name},
};

pub mod assert;
pub mod utils;

#[test]
fn decodes_different_sample_rates_and_channels() {
    let output_sample_rates = &*SUPPORTED_SAMPLE_RATES;
    let output_channel_counts = &*SUPPORTED_CHANNEL_COUNTS;
    let sample_rates = &*SUPPORTED_SAMPLE_RATES;
    let channel_counts = &*SUPPORTED_CHANNEL_COUNTS;
    let formats = &*SUPPORTED_FILE_FORMATS;

    let mut test_cases = Vec::new();

    for output_sample_rate in output_sample_rates.iter() {
        for output_channels in output_channel_counts.iter() {
            for input_sample_rate in sample_rates.iter() {
                for input_channels in channel_counts.iter() {
                    for format in formats.iter() {
                        let input_sample_rate = *input_sample_rate;
                        let input_channels = *input_channels;
                        let output_sample_rate = *output_sample_rate;
                        let output_channels = *output_channels;
                        let format = *format;

                        let test_case = (
                            input_sample_rate,
                            input_channels,
                            output_sample_rate,
                            output_channels,
                            format,
                        );

                        test_cases.push(test_case);
                    }
                }
            }
        }
    }

    test_cases.par_iter().for_each(
        |&(input_sample_rate, input_channels, output_sample_rate, output_channels, format)| {
            test_playback_controller_play(
                input_sample_rate,
                input_channels,
                output_sample_rate,
                output_channels,
                format.to_owned(),
            );
        },
    );
}

fn test_playback_controller_play(
    input_sample_rate: u32,
    input_channels: u16,
    output_sample_rate: u32,
    output_channels: u16,
    format: String,
) {
    let mut playback = TestPlayback::build(output_sample_rate, output_channels);

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    let mut sample_count = 0;
    let total_output_samples = output_sample_rate as usize * output_channels as usize;

    let file_name = get_file_name(input_sample_rate, input_channels, &format);
    let file_path = audio_file_fixtures_path
        .all_sample_rates_and_channels
        .join(&file_name);
    let mock_track = create_mock_track(file_path);

    playback.playback_controller.play(mock_track).unwrap();

    let mut playback_engine = playback.playback_engine.borrow_mut();

    loop {
        let sample_buffer_consumer = playback_engine.sample_buffer_consumer.as_mut().unwrap();

        if sample_buffer_consumer.is_empty() {
            if let Ok(event) = playback.playback_controller.poll_audio_pipeline_event() {
                match event {
                    Some(AudioPipelineThreadEvent::TrackFinished) => break,
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

    assert_sample_count(sample_count, total_output_samples, &file_name, None);
}

#[test]
fn performs_seek_correctly() {
    let input_sample_rate = 48000;
    let input_channels = 2;
    let output_sample_rate = 44100;
    let output_channels = 2;

    let formats = &*SUPPORTED_FILE_FORMATS;

    let mut test_cases = Vec::new();

    for format in formats.iter() {
        let format = *format;

        let test_case = (
            input_sample_rate,
            input_channels,
            output_sample_rate,
            output_channels,
            format,
        );

        test_cases.push(test_case);
    }

    test_cases.par_iter().for_each(
        |&(input_sample_rate, input_channels, output_sample_rate, output_channels, format)| {
            test_playback_controller_seek(
                input_sample_rate,
                input_channels,
                output_sample_rate,
                output_channels,
                format.to_owned(),
            );
        },
    );
}

fn test_playback_controller_seek(
    input_sample_rate: u32,
    input_channels: u16,
    output_sample_rate: u32,
    output_channels: u16,
    format: String,
) {
    let mut playback = TestPlayback::build(output_sample_rate, output_channels);

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    let mut sample_count = 0;
    let total_track_frames = input_sample_rate as usize;
    let total_output_samples = output_sample_rate as usize * output_channels as usize;

    let file_name = get_file_name(input_sample_rate, input_channels, &format);
    let file_path = audio_file_fixtures_path
        .all_sample_rates_and_channels
        .join(&file_name);
    let mock_track = create_mock_track(file_path);

    playback.playback_controller.play(mock_track).unwrap();

    playback.playback_controller.stop().unwrap();

    playback
        .playback_controller
        .seek(total_track_frames as u64 / 2)
        .unwrap();

    std::thread::sleep(Duration::from_millis(10));

    let mut playback_engine = playback.playback_engine.borrow_mut();

    loop {
        let sample_buffer_consumer = playback_engine.sample_buffer_consumer.as_mut().unwrap();

        if sample_buffer_consumer.is_empty() {
            if let Ok(event) = playback.playback_controller.poll_audio_pipeline_event() {
                match event {
                    Some(AudioPipelineThreadEvent::TrackFinished) => break,
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

    match format.as_ref() {
        "mp3" => assert_sample_count(sample_count, total_output_samples / 2, &file_name, Some(15)), // 15% of tolerance for mp3
        _ => assert_sample_count(sample_count, total_output_samples / 2, &file_name, Some(5)), // 5% of tolerance by default
    };
}
