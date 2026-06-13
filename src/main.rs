use std::process::exit;

use iced_aw::ICED_AW_FONT_BYTES;
use nameless_music_player_lib::{
    app::App,
    database::initialize_database,
    log::initialize_logging,
    playback::{PlaybackController, engine::AudioEngine},
    ui::theme::Theme,
};
use rfd::{MessageDialog, MessageLevel};
use tracing::{error, info};

fn main() -> iced::Result {
    #[cfg(debug_assertions)]
    let _ = dotenvy::dotenv();

    let _worker_guard = initialize_logging();

    info!(version = env!("CARGO_PKG_VERSION"), "Starting application.");

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let pool = match runtime.block_on(initialize_database()) {
        Ok(pool) => pool,
        Err(error) => {
            error!("Failed to initialize database {:?}", error);

            MessageDialog::new()
                .set_title("Nameless Music Player")
                .set_description(format!("Something went wrong while initializing the app."))
                .set_level(MessageLevel::Error)
                .show();

            exit(1);
        }
    };

    let mut playback_controller = PlaybackController::new(Box::new(AudioEngine::new()));

    playback_controller.initialize_output().unwrap();

    iced::application(
        move || {
            let mut playback_controller = PlaybackController::new(Box::new(AudioEngine::new()));

            playback_controller.initialize_output().unwrap();

            App::new(pool.clone(), Theme::default(), 1.0, playback_controller)
        },
        App::update,
        App::view,
    )
    .title(App::title)
    .theme(App::theme)
    .scale_factor(|app: &App| app.scale_factor())
    .window_size((1024.0, 768.0))
    .font(ICED_AW_FONT_BYTES)
    .font(include_bytes!("../fonts/music-player-icons.ttf"))
    .run()
}
