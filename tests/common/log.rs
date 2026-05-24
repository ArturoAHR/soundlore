use std::sync::Once;

use tracing_subscriber::{fmt, EnvFilter};

static INITIALIZATION_LOCK: Once = Once::new();

pub fn initialize_logging() {
    INITIALIZATION_LOCK.call_once(|| {
        let _ = dotenvy::dotenv();

        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("debug,nameless_music_player=trace,sqlx=warn,iced=warn")
        });

        let _ = fmt()
            .with_env_filter(filter)
            .with_test_writer()
            .with_target(true)
            .with_line_number(true)
            .try_init();
    });
}
