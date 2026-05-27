use std::sync::LazyLock;

use crate::common::models::ExpectedTrack;

pub static ALL_FORMATS_EXPECTED_TRACKS: LazyLock<Vec<ExpectedTrack>> = LazyLock::new(|| {
    vec![
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
    ]
});

pub static METADATA_VARIANTS_EXPECTED_TRACKS: LazyLock<Vec<ExpectedTrack>> = LazyLock::new(|| {
    vec![
        ExpectedTrack {
            file_name: "only_title.ogg".to_owned(),
            file_format: "ogg".to_owned(),

            // Codec Parameters
            codec: "vorbis".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("Only Title".to_owned()),
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
            file_name: "artist_album.ogg".to_owned(),
            file_format: "ogg".to_owned(),

            // Codec Parameters
            codec: "vorbis".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: None,
            artist: Some("Solo Artist".to_owned()),
            album: Some("Solo Album".to_owned()),
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
            file_name: "unicode_title.flac".to_owned(),
            file_format: "flac".to_owned(),

            // Codec Parameters
            codec: "flac".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("日本語タイトル ⟨long⟩".to_owned()),
            artist: Some(format!("   {}", "x".repeat(2048))),
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
            file_name: "no_tags.ogg".to_owned(),
            file_format: "ogg".to_owned(),

            // Codec Parameters
            codec: "vorbis".to_owned(),
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
            file_name: "slash_numerics.flac".to_owned(),
            file_format: "flac".to_owned(),

            // Codec Parameters
            codec: "flac".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("Slash Numerics".to_owned()),
            artist: None,
            album: None,
            album_artist: None,
            track_number: Some(3),
            track_total: Some(12),
            disc_number: Some(1),
            disc_total: Some(2),
            year: Some(2020),
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
            file_name: "unicode_mp3.mp3".to_owned(),
            file_format: "mp3".to_owned(),

            // Codec Parameters
            codec: "mp3".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("日本語タイトル ⟨long⟩".to_owned()),
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
            file_name: "unicode_ogg.ogg".to_owned(),
            file_format: "ogg".to_owned(),

            // Codec Parameters
            codec: "vorbis".to_owned(),
            sample_rate: 44_100,
            channels: 1,

            // Track Metadata
            title: Some("日本語タイトル ⟨long⟩".to_owned()),
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
    ]
});

pub static CORRUPT_EXPECTED_TRACKS: LazyLock<Vec<ExpectedTrack>> = LazyLock::new(|| {
    vec![ExpectedTrack {
        file_name: "mislabeled.mp3".to_owned(),
        file_format: "mp3".to_owned(),

        // Codec Parameters
        codec: "vorbis".to_owned(),
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
    }]
});

pub static PARTIALLY_CORRUPT_EXPECTED_TRACKS: LazyLock<Vec<ExpectedTrack>> = LazyLock::new(|| {
    vec![
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
    ]
});
