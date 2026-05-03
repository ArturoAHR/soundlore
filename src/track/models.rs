use sea_query::enum_def;

#[enum_def(table_name = "tracks")]
#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Track {
    pub id: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<i64>,
    pub disc_number: Option<i64>,
    pub year: Option<i64>,
    pub genre: Option<String>,
    pub duration_secs: Option<f64>,
    pub bitrate: Option<i64>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub format: Option<String>,
    pub play_count: i64,
    pub skip_count: i64,
    pub volume_adjustment: f64,
    pub last_played: Option<i64>,
    pub rating: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Default)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<i64>,
    pub disc_number: Option<i64>,
    pub year: Option<i64>,
    pub genre: Option<String>,
    pub duration_secs: Option<f64>,
    pub bitrate: Option<i64>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub format: Option<String>,
}

#[derive(Debug)]
pub struct UpdateTrack {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<i64>,
    pub disc_number: Option<i64>,
    pub year: Option<i64>,
    pub genre: Option<String>,
    pub duration_secs: Option<f64>,
    pub bitrate: Option<i64>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
    pub file_size: Option<i64>,
    pub format: Option<String>,
    pub play_count: Option<i64>,
    pub skip_count: Option<i64>,
    pub volume_adjustment: Option<f64>,
    pub last_played: Option<i64>,
    pub rating: Option<i64>,
    pub deleted_at: Option<i64>,
}
