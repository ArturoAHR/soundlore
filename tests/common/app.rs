use nameless_music_player_lib::{
    app::{App, Message},
    ui::theme::Theme,
};
use sqlx::SqlitePool;

use crate::common::{
    database::get_database_pool, emulation::perform_task, log::initialize_logging,
};

pub struct TestApp {
    pub app: App,
    pub pool: SqlitePool,
}

impl TestApp {
    pub async fn build() -> Self {
        initialize_logging();

        let pool = get_database_pool().await;

        let (mut app, initial_task) = App::new(pool.clone(), Theme::DARK, 1.0);

        perform_task(&mut app, initial_task).await;

        Self { app, pool }
    }

    pub async fn dispatch_message(&mut self, message: Message) {
        let task = self.app.update(message);

        perform_task(&mut self.app, task).await;
    }

    pub fn state(&self) -> &App {
        &self.app
    }
}
