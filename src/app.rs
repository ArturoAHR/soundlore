use log::error;
use sqlx::SqlitePool;
use std::fmt;

use iced::{
    widget::{center, text},
    Element, Task,
};

use crate::core::database::initialize_database;

#[derive(Debug)]
pub enum App {
    Loading,
    Ready(State),
}

pub struct State {
    pool: SqlitePool,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyStruct").finish()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Ready(Result<SqlitePool, String>),
    // ScanLibrary,
    // ScanComplete(Result<(), String>),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            App::Loading,
            Task::perform(async { initialize_database().await }, |result| {
                Message::Ready(result.map_err(|e| e.to_string()))
            }),
        )
    }
    pub fn title(&self) -> String {
        String::from("Nameless Music Player")
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ready(Ok(pool)) => {
                *self = App::Ready(State { pool });
                Task::none()
            }
            Message::Ready(Err(error)) => {
                error!("Failed to connect with database: {:?}", error);
                Task::none()
            }
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        center(text("Nameless Music Player").size(28)).into()
    }
}
