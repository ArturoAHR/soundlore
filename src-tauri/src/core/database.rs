use tauri::Manager;

use crate::config::DATABASE_FILE_NAME;

pub fn get_database_path(app: &tauri::AppHandle) -> String {
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");

    format!("sqlite:{}", data_dir.join(DATABASE_FILE_NAME).display())
}
