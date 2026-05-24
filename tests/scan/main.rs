use nameless_music_player_lib::app::{AppStatus, Message};
use pretty_assertions::assert_eq;
use sqlx::query_scalar;

use crate::common::{app::TestApp, file::AUDIO_FILE_FIXTURES_PATH};

#[path = "../common/mod.rs"]
mod common;

#[tokio::test]
async fn scans_successfully_all_supported_formats() {
    let mut app = TestApp::build().await;

    let audio_file_fixtures_path = &*AUDIO_FILE_FIXTURES_PATH;

    app.dispatch_message(Message::ScanDirectory(Some(vec![audio_file_fixtures_path
        .all_formats
        .clone()])))
        .await;

    let count: i64 = query_scalar("SELECT COUNT(*) FROM track")
        .fetch_one(&app.pool)
        .await
        .unwrap();

    println!("{}", count);
    assert!(count > 0);

    assert_eq!(app.state().status, AppStatus::FinishedAddingTracks);
}
