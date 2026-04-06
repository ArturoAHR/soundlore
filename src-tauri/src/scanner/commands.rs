use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::scanner::service;

#[tauri::command]
pub async fn scan_files_in_directory(
    pool: State<'_, SqlitePool>,
    directories: Vec<String>,
) -> Result<(), AppError> {
    service::scan_files_in_directory(&pool, directories).await?;

    Ok(())
}
