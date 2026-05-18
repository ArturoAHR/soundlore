use nameless_music_player_lib::app::App;

fn main() -> iced::Result {
    env_logger::init();

    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .window_size((1024.0, 768.0))
        .font(include_bytes!("../fonts/music-player-icons.ttf"))
        .run()
}
