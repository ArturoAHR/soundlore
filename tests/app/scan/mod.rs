use sea_query::{Asterisk, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use soundlore_lib::{
    app::{AppStatus, Event},
    track::models::{Track, TrackIden},
};

use crate::{
    common::{app::TestApp, assert::assert_tracks, file::AUDIO_FILE_FIXTURES_PATH},
    scan::constants::{
        ALL_FORMATS_EXPECTED_TRACKS, CORRUPT_EXPECTED_TRACKS, METADATA_VARIANTS_EXPECTED_TRACKS,
        PARTIALLY_CORRUPT_EXPECTED_TRACKS,
    },
};

mod constants;

#[tokio::test]
async fn scans_successfully_all_supported_formats() {
    let mut app = TestApp::build().await;

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    app.dispatch_message(Event::ScanDirectory(Some(vec![
        audio_file_fixtures_path.all_formats.clone(),
    ])))
    .await;

    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(TrackIden::Table)
        .build_sqlx(SqliteQueryBuilder);

    let tracks: Vec<Track> = sqlx::query_as_with(&sql, values)
        .fetch_all(&app.pool)
        .await
        .unwrap();

    assert_tracks(&ALL_FORMATS_EXPECTED_TRACKS, &tracks);

    assert!(matches!(
        app.state().status,
        AppStatus::FinishedAddingTracks
    ));
}

#[tokio::test]
async fn scans_successfully_tracks_with_varying_metadata() {
    let mut app = TestApp::build().await;

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    app.dispatch_message(Event::ScanDirectory(Some(vec![
        audio_file_fixtures_path.metadata_variants.clone(),
    ])))
    .await;

    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(TrackIden::Table)
        .build_sqlx(SqliteQueryBuilder);

    let tracks: Vec<Track> = sqlx::query_as_with(&sql, values)
        .fetch_all(&app.pool)
        .await
        .unwrap();

    assert_tracks(&METADATA_VARIANTS_EXPECTED_TRACKS, &tracks);

    assert!(matches!(
        app.state().status,
        AppStatus::FinishedAddingTracks
    ));
}

#[tokio::test]
async fn scans_successfully_corrupt_tracks() {
    let mut app = TestApp::build().await;

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    app.dispatch_message(Event::ScanDirectory(Some(vec![
        audio_file_fixtures_path.corrupt.clone(),
    ])))
    .await;

    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(TrackIden::Table)
        .build_sqlx(SqliteQueryBuilder);

    let tracks: Vec<Track> = sqlx::query_as_with(&sql, values)
        .fetch_all(&app.pool)
        .await
        .unwrap();

    assert_tracks(&CORRUPT_EXPECTED_TRACKS, &tracks);

    assert!(matches!(
        app.state().status,
        AppStatus::FinishedAddingTracks
    ));
}

#[tokio::test]
async fn scans_successfully_partially_corrupt_tracks() {
    let mut app = TestApp::build().await;

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    app.dispatch_message(Event::ScanDirectory(Some(vec![
        audio_file_fixtures_path.partially_corrupt.clone(),
    ])))
    .await;

    let (sql, values) = Query::select()
        .column(Asterisk)
        .from(TrackIden::Table)
        .build_sqlx(SqliteQueryBuilder);

    let tracks: Vec<Track> = sqlx::query_as_with(&sql, values)
        .fetch_all(&app.pool)
        .await
        .unwrap();

    assert_tracks(&PARTIALLY_CORRUPT_EXPECTED_TRACKS, &tracks);

    assert!(matches!(
        app.state().status,
        AppStatus::FinishedAddingTracks
    ));
}
