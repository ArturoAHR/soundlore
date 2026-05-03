use std::sync::Arc;

use async_trait::async_trait;
use sea_query::{OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::AppError;
use crate::track::models::{TrackIden, TrackMetadata};

#[async_trait]
pub trait TrackRepository: Send + Sync {
    async fn upsert_track(&self, track: &TrackMetadata) -> Result<(), AppError>;
}

pub struct TrackRepositoryImpl {
    pool: Arc<SqlitePool>,
}

impl TrackRepositoryImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TrackRepository for TrackRepositoryImpl {
    async fn upsert_track(&self, track: &TrackMetadata) -> Result<(), AppError> {
        let id = Uuid::new_v4();

        let (sql, values) = Query::insert()
            .into_table(TrackIden::Table)
            .columns([
                TrackIden::Id,
                TrackIden::FilePath,
                TrackIden::Title,
                TrackIden::Artist,
                TrackIden::Album,
                TrackIden::AlbumArtist,
                TrackIden::TrackNumber,
                TrackIden::DiscNumber,
                TrackIden::Year,
                TrackIden::Genre,
                TrackIden::DurationSecs,
                TrackIden::Bitrate,
                TrackIden::SampleRate,
                TrackIden::Channels,
                TrackIden::FileSize,
                TrackIden::Format,
            ])
            .values([
                id.to_string().into(),
                track.file_path.clone().into(),
                track.title.clone().into(),
                track.artist.clone().into(),
                track.album.clone().into(),
                track.album_artist.clone().into(),
                track.track_number.into(),
                track.disc_number.into(),
                track.year.into(),
                track.genre.clone().into(),
                track.duration_secs.into(),
                track.bitrate.into(),
                track.sample_rate.into(),
                track.channels.into(),
                track.file_size.into(),
                track.format.clone().into(),
            ])?
            .on_conflict(
                OnConflict::column(TrackIden::FilePath)
                    .update_columns([
                        TrackIden::Title,
                        TrackIden::Artist,
                        TrackIden::Album,
                        TrackIden::AlbumArtist,
                        TrackIden::TrackNumber,
                        TrackIden::DiscNumber,
                        TrackIden::Year,
                        TrackIden::Genre,
                        TrackIden::DurationSecs,
                        TrackIden::Bitrate,
                        TrackIden::SampleRate,
                        TrackIden::Channels,
                        TrackIden::FileSize,
                        TrackIden::Format,
                    ])
                    .to_owned(),
            )
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }
}
