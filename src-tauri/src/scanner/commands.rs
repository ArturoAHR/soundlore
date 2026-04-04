use crate::error::AppError;
use crate::scanner::service;

#[tauri::command]
pub fn scan_files_in_directory(directories: Vec<String>) -> Result<(), AppError> {
    service::scan_files_in_directory(directories)?;

    Ok(())
}
