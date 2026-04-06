mod config;
mod error;

mod core;
mod file;
mod scanner;
mod track;

use tauri::Manager;

use crate::core::database::{check_schema_version, create_pool};
use crate::core::migrations::run_migrations;
use crate::scanner::commands::scan_files_in_directory;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let verbose = std::env::args().any(|a| a == "--verbose" || a == "-v");
    let very_verbose = std::env::args().any(|a| a == "--very-verbose" || a == "-vv");

    let mut log_level = tauri_plugin_log::log::LevelFilter::Info;

    if verbose {
        log_level = tauri_plugin_log::log::LevelFilter::Debug;
    }

    if very_verbose {
        log_level = tauri_plugin_log::log::LevelFilter::Trace;
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log_level)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .build(),
        )
        .setup(|app| {
            tauri::async_runtime::block_on(async {
                let pool = create_pool(&app.handle()).await?;

                check_schema_version(&pool).await?;
                run_migrations(&pool).await?;

                app.manage(pool);

                Ok(())
            })
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, scan_files_in_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
