mod config;
mod error;

mod core;
mod scanner;

use crate::core::migrations::run_migrations;
use crate::scanner::commands::scan_files_in_directory;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| tauri::async_runtime::block_on(run_migrations(&app.handle())))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_files_in_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
