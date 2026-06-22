use std::{error::Error, fs, io::stderr};

use tracing::error;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn initialize_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_directory = dirs::data_local_dir()?.join("soundlore").join("logs");
    fs::create_dir_all(&log_directory).ok()?;

    let file_appender = tracing_appender::rolling::daily(&log_directory, "app.log");
    let (non_blocking_writer, worker_guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_filter(get_filter());

    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr)
        .with_ansi(true)
        .with_filter(get_filter());

    let registry = tracing_subscriber::registry().with(file_layer);

    #[cfg(debug_assertions)]
    let registry = registry.with(console_layer);

    registry.init();

    Some(worker_guard)
}

fn get_filter() -> EnvFilter {
    tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "debug,soundlore=debug,sqlx=warn,wgpu=warn,iced=info,naga=warn,winit=warn,cosmic_text=warn,sctk=warn".into()
        } else {
            "warn,soundlore=info".into()
        }
    })
}

pub fn log_error_chain(error: &(dyn Error + 'static)) {
    error!("{error}");

    let mut source = error.source();
    while let Some(current_error) = source {
        error!("  caused by: {current_error}");
        source = current_error.source();
    }
}
