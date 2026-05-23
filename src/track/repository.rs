use sea_query::{OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

use sqlx::SqlitePool;
use tracing::instrument;
use uuid::Uuid;

use crate::error::AppError;
use crate::track::models::{TrackIden, TrackProperties};

#[instrument(skip(pool))]
pub async fn upsert_track(pool: &SqlitePool, track: &TrackProperties) -> Result<(), AppError> {
    let id = Uuid::new_v4();

    let (sql, values) = Query::insert()
        .into_table(TrackIden::Table)
        .columns([
            TrackIden::Id,
            // File Metadata
            TrackIden::FilePath,
            TrackIden::FileSizeBytes,
            TrackIden::FileFormat,
            // Codec Parameters
            TrackIden::Codec,
            TrackIden::DurationSecs,
            TrackIden::SampleRate,
            TrackIden::Channels,
            TrackIden::BitDepth,
            TrackIden::BitrateKbps,
            // Track Metadata
            TrackIden::Title,
            TrackIden::Artist,
            TrackIden::Album,
            TrackIden::AlbumArtist,
            TrackIden::TrackNumber,
            TrackIden::TrackTotal,
            TrackIden::DiscNumber,
            TrackIden::DiscTotal,
            TrackIden::Year,
            TrackIden::Genre,
            // ReplayGain
            TrackIden::ReplaygainTrackGainDb,
            TrackIden::ReplaygainTrackPeak,
            TrackIden::ReplaygainAlbumGainDb,
            TrackIden::ReplaygainAlbumPeak,
        ])
        .values([
            id.to_string().into(),
            // File Metadata
            track.file_path.clone().into(),
            track.file_size_bytes.into(),
            track.file_format.clone().into(),
            // Audio Metadata
            track.codec.clone().into(),
            track.duration_secs.into(),
            track.sample_rate.into(),
            track.channels.into(),
            track.bit_depth.into(),
            track.bitrate_kbps.into(),
            // Track Metadata
            track.title.clone().into(),
            track.artist.clone().into(),
            track.album.clone().into(),
            track.album_artist.clone().into(),
            track.track_number.into(),
            track.track_total.into(),
            track.disc_number.into(),
            track.disc_total.into(),
            track.year.into(),
            track.genre.clone().into(),
            // ReplayGain
            track.replaygain_track_gain_db.into(),
            track.replaygain_track_peak.into(),
            track.replaygain_album_gain_db.into(),
            track.replaygain_album_peak.into(),
        ])?
        .on_conflict(
            OnConflict::column(TrackIden::FilePath)
                .update_columns([
                    // File Metadata
                    TrackIden::FileSizeBytes,
                    // Codec Parameters
                    TrackIden::Codec,
                    TrackIden::DurationSecs,
                    TrackIden::SampleRate,
                    TrackIden::Channels,
                    TrackIden::BitDepth,
                    TrackIden::BitrateKbps,
                    // Track Metadata
                    TrackIden::Title,
                    TrackIden::Artist,
                    TrackIden::Album,
                    TrackIden::AlbumArtist,
                    TrackIden::TrackNumber,
                    TrackIden::TrackTotal,
                    TrackIden::DiscNumber,
                    TrackIden::DiscTotal,
                    TrackIden::Year,
                    TrackIden::Genre,
                    // ReplayGain
                    TrackIden::ReplaygainTrackGainDb,
                    TrackIden::ReplaygainTrackPeak,
                    TrackIden::ReplaygainAlbumGainDb,
                    TrackIden::ReplaygainAlbumPeak,
                ])
                .to_owned(),
        )
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

#[instrument(skip(pool))]
pub async fn upsert_tracks_batch(
    pool: &SqlitePool,
    tracks: &[TrackProperties],
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;

    for track in tracks {
        let id = Uuid::new_v4();

        let (sql, values) = Query::insert()
            .into_table(TrackIden::Table)
            .columns([
                TrackIden::Id,
                // File Metadata
                TrackIden::FilePath,
                TrackIden::FileSizeBytes,
                TrackIden::FileFormat,
                // Codec Parameters
                TrackIden::Codec,
                TrackIden::DurationSecs,
                TrackIden::SampleRate,
                TrackIden::Channels,
                TrackIden::BitDepth,
                TrackIden::BitrateKbps,
                // Track Metadata
                TrackIden::Title,
                TrackIden::Artist,
                TrackIden::Album,
                TrackIden::AlbumArtist,
                TrackIden::TrackNumber,
                TrackIden::TrackTotal,
                TrackIden::DiscNumber,
                TrackIden::DiscTotal,
                TrackIden::Year,
                TrackIden::Genre,
                // ReplayGain
                TrackIden::ReplaygainTrackGainDb,
                TrackIden::ReplaygainTrackPeak,
                TrackIden::ReplaygainAlbumGainDb,
                TrackIden::ReplaygainAlbumPeak,
            ])
            .values([
                id.to_string().into(),
                // File Metadata
                track.file_path.clone().into(),
                track.file_size_bytes.into(),
                track.file_format.clone().into(),
                // Audio Metadata
                track.codec.clone().into(),
                track.duration_secs.into(),
                track.sample_rate.into(),
                track.channels.into(),
                track.bit_depth.into(),
                track.bitrate_kbps.into(),
                // Track Metadata
                track.title.clone().into(),
                track.artist.clone().into(),
                track.album.clone().into(),
                track.album_artist.clone().into(),
                track.track_number.into(),
                track.track_total.into(),
                track.disc_number.into(),
                track.disc_total.into(),
                track.year.into(),
                track.genre.clone().into(),
                // ReplayGain
                track.replaygain_track_gain_db.into(),
                track.replaygain_track_peak.into(),
                track.replaygain_album_gain_db.into(),
                track.replaygain_album_peak.into(),
            ])?
            .on_conflict(
                OnConflict::column(TrackIden::FilePath)
                    .update_columns([
                        // File Metadata
                        TrackIden::FileSizeBytes,
                        // Codec Parameters
                        TrackIden::Codec,
                        TrackIden::DurationSecs,
                        TrackIden::SampleRate,
                        TrackIden::Channels,
                        TrackIden::BitDepth,
                        TrackIden::BitrateKbps,
                        // Track Metadata
                        TrackIden::Title,
                        TrackIden::Artist,
                        TrackIden::Album,
                        TrackIden::AlbumArtist,
                        TrackIden::TrackNumber,
                        TrackIden::TrackTotal,
                        TrackIden::DiscNumber,
                        TrackIden::DiscTotal,
                        TrackIden::Year,
                        TrackIden::Genre,
                        // ReplayGain
                        TrackIden::ReplaygainTrackGainDb,
                        TrackIden::ReplaygainTrackPeak,
                        TrackIden::ReplaygainAlbumGainDb,
                        TrackIden::ReplaygainAlbumPeak,
                    ])
                    .to_owned(),
            )
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *transaction)
            .await?;
    }

    transaction.commit().await?;

    Ok(())
}
