use std::path::PathBuf;

use soundlore_lib::track::{file::read_track_properties, models::Track};

pub fn create_mock_track(track_path: PathBuf) -> Track {
    let track_properties = read_track_properties(&track_path).unwrap();

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
