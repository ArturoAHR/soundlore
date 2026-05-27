use nameless_music_player_lib::{
    app::{AppStatus, Message},
    track::models::{Track, TrackIden},
};
use pretty_assertions::assert_eq;
use sea_query::{Asterisk, Expr, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::query_scalar;

use crate::common::{app::TestApp, file::AUDIO_FILE_FIXTURES_PATH};

#[path = "../common/mod.rs"]
mod common;

struct ExpectedTrack {
    // File Metadata
    pub file_name: String,
    pub file_format: String,

    // Codec Parameters
    pub codec: String,
    pub sample_rate: i64,
    pub channels: i64,

    // Track Metadata
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<i64>,
    pub track_total: Option<i64>,
    pub disc_number: Option<i64>,
    pub disc_total: Option<i64>,
    pub year: Option<i64>,
    pub genre: Option<String>,

    // ReplayGain
    pub replaygain_track_gain_db: Option<f64>,
    pub replaygain_track_peak: Option<f64>,
    pub replaygain_album_gain_db: Option<f64>,
    pub replaygain_album_peak: Option<f64>,

    // Playback
    pub play_count: i64,
    pub skip_count: i64,
    pub volume_adjustment_db: f64,
    pub last_played: Option<i64>,
}

#[tokio::test]
async fn scans_successfully_all_supported_formats() {
    let mut app = TestApp::build().await;

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    app.dispatch_message(Message::ScanDirectory(Some(vec![audio_file_fixtures_path
        .all_formats
        .clone()])))
        .await;

    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(TrackIden::Table)
        .build_sqlx(SqliteQueryBuilder);

    let tracks: Vec<Track> = sqlx::query_as_with(&sql, values)
        .fetch_all(&app.pool)
        .await
        .unwrap();

    let expected_tracks = vec![
        ExpectedTrack {
            file_name: "track.ogg".to_owned(),
            file_format: "ogg".to_owned(),

            // Codec Parameters
            codec: "vorbis".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("Test Track".to_owned()),
            artist: Some("Test Artist".to_owned()),
            album: Some("Test Album".to_owned()),
            album_artist: Some("Test Album Artist".to_owned()),
            track_number: Some(3),
            track_total: Some(12),
            disc_number: Some(1),
            disc_total: Some(2),
            year: Some(2024),
            genre: Some("Test Genre".to_owned()),

            // ReplayGain
            replaygain_track_gain_db: Some(-6.54),
            replaygain_track_peak: Some(0.987654),
            replaygain_album_gain_db: Some(-7.2),
            replaygain_album_peak: Some(0.999),

            // Playback
            play_count: 0,
            skip_count: 0,
            volume_adjustment_db: 0.0,
            last_played: None,
        },
        ExpectedTrack {
            file_name: "track.aiff".to_owned(),
            file_format: "aiff".to_owned(),

            // Codec Parameters
            codec: "pcm_s16be".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("Test Track".to_owned()),
            artist: Some("Test Artist".to_owned()),
            album: Some("Test Album".to_owned()),
            album_artist: Some("Test Album Artist".to_owned()),
            track_number: Some(3),
            track_total: Some(12),
            disc_number: Some(1),
            disc_total: Some(2),
            year: Some(2024),
            genre: Some("Test Genre".to_owned()),

            // ReplayGain
            replaygain_track_gain_db: Some(-6.54),
            replaygain_track_peak: Some(0.987654),
            replaygain_album_gain_db: Some(-7.2),
            replaygain_album_peak: Some(0.999),

            // Playback
            play_count: 0,
            skip_count: 0,
            volume_adjustment_db: 0.0,
            last_played: None,
        },
        ExpectedTrack {
            file_name: "track.mp3".to_owned(),
            file_format: "mp3".to_owned(),

            // Codec Parameters
            codec: "mp3".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("Test Track".to_owned()),
            artist: Some("Test Artist".to_owned()),
            album: Some("Test Album".to_owned()),
            album_artist: Some("Test Album Artist".to_owned()),
            track_number: Some(3),
            track_total: Some(12),
            disc_number: Some(1),
            disc_total: Some(2),
            year: Some(2024),
            genre: Some("Test Genre".to_owned()),

            // ReplayGain
            replaygain_track_gain_db: Some(-6.54),
            replaygain_track_peak: Some(0.987654),
            replaygain_album_gain_db: Some(-7.2),
            replaygain_album_peak: Some(0.999),

            // Playback
            play_count: 0,
            skip_count: 0,
            volume_adjustment_db: 0.0,
            last_played: None,
        },
        ExpectedTrack {
            file_name: "track.flac".to_owned(),
            file_format: "flac".to_owned(),

            // Codec Parameters
            codec: "flac".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("Test Track".to_owned()),
            artist: Some("Test Artist".to_owned()),
            album: Some("Test Album".to_owned()),
            album_artist: Some("Test Album Artist".to_owned()),
            track_number: Some(3),
            track_total: Some(12),
            disc_number: Some(1),
            disc_total: Some(2),
            year: Some(2024),
            genre: Some("Test Genre".to_owned()),

            // ReplayGain
            replaygain_track_gain_db: Some(-6.54),
            replaygain_track_peak: Some(0.987654),
            replaygain_album_gain_db: Some(-7.2),
            replaygain_album_peak: Some(0.999),

            // Playback
            play_count: 0,
            skip_count: 0,
            volume_adjustment_db: 0.0,
            last_played: None,
        },
        ExpectedTrack {
            file_name: "track.m4a".to_owned(),
            file_format: "m4a".to_owned(),

            // Codec Parameters
            codec: "aac".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("Test Track".to_owned()),
            artist: Some("Test Artist".to_owned()),
            album: Some("Test Album".to_owned()),
            album_artist: Some("Test Album Artist".to_owned()),
            track_number: Some(3),
            track_total: Some(12),
            disc_number: Some(1),
            disc_total: Some(2),
            year: Some(2024),
            genre: Some("Test Genre".to_owned()),

            // ReplayGain
            replaygain_track_gain_db: None,
            replaygain_track_peak: None,
            replaygain_album_gain_db: None,
            replaygain_album_peak: None,

            // Playback
            play_count: 0,
            skip_count: 0,
            volume_adjustment_db: 0.0,
            last_played: None,
        },
        ExpectedTrack {
            file_name: "track.wav".to_owned(),
            file_format: "wav".to_owned(),

            // Codec Parameters
            codec: "pcm_s16le".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            track_total: None,
            disc_number: None,
            disc_total: None,
            year: None,
            genre: None,

            // ReplayGain
            replaygain_track_gain_db: None,
            replaygain_track_peak: None,
            replaygain_album_gain_db: None,
            replaygain_album_peak: None,

            // Playback
            play_count: 0,
            skip_count: 0,
            volume_adjustment_db: 0.0,
            last_played: None,
        },
        ExpectedTrack {
            file_name: "track.aac".to_owned(),
            file_format: "aac".to_owned(),

            // Codec Parameters
            codec: "aac".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            track_number: None,
            track_total: None,
            disc_number: None,
            disc_total: None,
            year: None,
            genre: None,

            // ReplayGain
            replaygain_track_gain_db: None,
            replaygain_track_peak: None,
            replaygain_album_gain_db: None,
            replaygain_album_peak: None,

            // Playback
            play_count: 0,
            skip_count: 0,
            volume_adjustment_db: 0.0,
            last_played: None,
        },
    ];

    for expected_track in expected_tracks {
        let track = tracks
            .iter()
            .find(|track| track.file_path.contains(&expected_track.file_name))
            .expect(&format!(
                "Could not find track {}",
                expected_track.file_name
            ));

        println!("{:?}", track);

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

    assert!(matches!(
        app.state().status,
        AppStatus::FinishedAddingTracks
    ));
}
