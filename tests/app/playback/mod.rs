use std::{path::PathBuf, thread, time::Duration};

use nameless_music_player_lib::{
    playback::pipeline::thread::AudioPipelineThreadEvent,
    track::{metadata::read_track_metadata, models::Track},
};
use tracing::debug;

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
        .play(create_mock_track(
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
fn decodes_different_sample_rates_and_channels_with_44100_stereo_output() {
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
                    _ => unreachable!(),
                };

                let file_name = format!("{}_{}.{}", sample_rate, channel_count_name, format);

                playback
                    .playback_controller
                    .play(create_mock_track(
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

                debug!("Sample count for {file_name}: {sample_count}");

                assert!(
                    sample_count >= output_channels as usize * output_sample_rate as usize,
                    "Insufficient sample count for file: {file_name}",
                );
            }
        }
    }
}

#[test]
fn decodes_different_sample_rates_and_channels_with_48000_stereo_output() {
    let output_sample_rate = 48000;
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
                    _ => unreachable!(),
                };

                let file_name = format!("{}_{}.{}", sample_rate, channel_count_name, format);

                playback
                    .playback_controller
                    .play(create_mock_track(
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

                debug!("Sample count for {file_name}: {sample_count}");

                assert!(
                    sample_count >= output_channels as usize * output_sample_rate as usize,
                    "Insufficient sample count for file: {file_name}",
                );
            }
        }
    }
}

fn create_mock_track(track_path: PathBuf) -> Track {
    let track_properties = read_track_metadata(&track_path).unwrap();

    Track {
        id: "36a7839f-15d5-44dd-9971-0696236370e9".to_owned(),
        file_path: track_properties.file_path,
        file_size_bytes: track_properties.file_size_bytes,
        file_format: track_properties.file_format,
        codec: track_properties.codec,
        frames: track_properties.frames,
        sample_rate: track_properties.sample_rate,
        channels: track_properties.channels,
        bit_depth: track_properties.bit_depth,
        bitrate_kbps: track_properties.bitrate_kbps,
        title: track_properties.title,
        artist: track_properties.artist,
        album: track_properties.album,
        album_artist: track_properties.album_artist,
        track_number: track_properties.track_number,
        track_total: track_properties.track_total,
        disc_number: track_properties.disc_number,
        disc_total: track_properties.disc_total,
        year: track_properties.year,
        genre: track_properties.genre,
        replaygain_track_gain_db: track_properties.replaygain_track_gain_db,
        replaygain_track_peak: track_properties.replaygain_track_peak,
        replaygain_album_gain_db: track_properties.replaygain_album_gain_db,
        replaygain_album_peak: track_properties.replaygain_album_peak,
        play_count: 0,
        skip_count: 0,
        volume_adjustment_db: 0.0,
        last_played: None,
        created_at: 1,
        updated_at: 1,
        deleted_at: None,
    }
}
