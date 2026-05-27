use nameless_music_player_lib::track::models::Track;
use pretty_assertions::assert_eq;

use crate::common::models::ExpectedTrack;

pub fn assert_tracks(expected_tracks: &[ExpectedTrack], tracks: &[Track]) {
    assert_eq!(expected_tracks.len(), tracks.len());

    for expected_track in expected_tracks.iter() {
        let track = tracks
            .iter()
            .find(|track| track.file_path.contains(&expected_track.file_name))
            .expect(&format!(
                "Could not find track {}",
                expected_track.file_name
            ));

        assert_eq!(expected_track.file_format, track.file_format);
        assert_eq!(expected_track.codec, track.codec);
        assert_eq!(expected_track.sample_rate, track.sample_rate);
        assert_eq!(expected_track.channels, track.channels);
        assert_eq!(expected_track.title, track.title);
        assert_eq!(expected_track.artist, track.artist);
        assert_eq!(expected_track.album, track.album);
        assert_eq!(expected_track.album_artist, track.album_artist);
        assert_eq!(expected_track.track_number, track.track_number);
        assert_eq!(expected_track.track_total, track.track_total);
        assert_eq!(expected_track.disc_number, track.disc_number);
        assert_eq!(expected_track.disc_total, track.disc_total);
        assert_eq!(expected_track.year, track.year);
        assert_eq!(expected_track.genre, track.genre);
        assert_eq!(
            expected_track.replaygain_track_gain_db,
            track.replaygain_track_gain_db
        );
        assert_eq!(
            expected_track.replaygain_track_peak,
            track.replaygain_track_peak
        );
        assert_eq!(
            expected_track.replaygain_album_gain_db,
            track.replaygain_album_gain_db
        );
        assert_eq!(
            expected_track.replaygain_album_peak,
            track.replaygain_album_peak
        );
        assert_eq!(expected_track.play_count, track.play_count);
        assert_eq!(expected_track.skip_count, track.skip_count);
        assert_eq!(
            expected_track.volume_adjustment_db,
            track.volume_adjustment_db
        );
        assert_eq!(expected_track.last_played, track.last_played);
    }
}
