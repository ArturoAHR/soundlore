use crate::error::AppError;

#[tauri::command]
pub fn scan_files_in_directory(directories: Vec<String>) -> Result<(), AppError> {
    print!("{:#?}", directories);

    Ok(())
}
