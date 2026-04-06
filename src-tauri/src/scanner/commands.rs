use tauri::State;

use crate::context::Context;
use crate::error::AppError;

#[tauri::command]
pub async fn scan_files_in_directory(
    ctx: State<'_, Context>,
    directories: Vec<String>,
) -> Result<(), AppError> {
    println!("{:#?}", directories);

    ctx.scanner_service
        .scan_files_in_directory(directories)
        .await?;

    Ok(())
}
