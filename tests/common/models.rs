pub struct ExpectedTrack {
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
