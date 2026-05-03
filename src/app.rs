use std::fmt;

use iced::{
    widget::{center, text},
    Element, Task,
};
use sqlx::SqlitePool;

use crate::{error::AppError, services::Services};

#[derive(Debug)]
pub enum App {
    Loaded,
}

pub struct State {
    services: Services,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyStruct")
            // simply omit `secret`
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Ready(Result<SqlitePool, String>),
    ScanLibrary,
    ScanComplete(Result<(), String>),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (App::Loaded, Task::none())
    }
    pub fn title(&self) -> String {
        String::from("Nameless Music Player")
    }
    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }
    pub fn view(&self) -> Element<'_, Message> {
        center(text("Nameless Music Player").size(28)).into()
    }
}
