use std::process::exit;

use iced_aw::ICED_AW_FONT_BYTES;
use log::error;
use nameless_music_player_lib::{app::App, database::initialize_database, ui::theme::Theme};
use rfd::{MessageDialog, MessageLevel};

fn main() -> iced::Result {
    env_logger::init();

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

    iced::application(
        move || App::new(pool.clone(), Theme::default(), 1.0),
        App::update,
        App::view,
    )
    .title(App::title)
    .theme(App::theme)
    .window_size((1024.0, 768.0))
    .font(ICED_AW_FONT_BYTES)
    .font(include_bytes!("../fonts/music-player-icons.ttf"))
    .scale_factor(|app: &App| app.scale_factor())
    .run()
}
