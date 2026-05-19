use log::error;
use sqlx::SqlitePool;

use iced::{
    widget::{button, center, column, container, row, space::horizontal, text},
    Element, Task,
};

use iced_aw::menu::{Item, Menu, MenuBar};

use crate::{
    database::initialize_database,
    ui::{
        icons::{self, icon},
        theme::{catalog::container::header, Theme},
    },
};

#[derive(Debug)]
pub enum App {
    Loading,
    Ready(State),
}

#[derive(Debug)]
pub struct State {
    pub pool: SqlitePool,
    pub ui_scale: f32,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub enum Message {
    Ready(Result<SqlitePool, String>),
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
                *self = App::Ready(State {
                    pool,
                    ui_scale: 1.0,
                    theme: Theme::DARK,
                });
                Task::none()
            }
            Message::Ready(Err(error)) => {
                error!("Failed to connect with database: {:?}", error);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        let dropdown = MenuBar::new(vec![Item::with_menu(
            button(icon(icons::MENU)),
            Menu::new(vec![
                Item::new(button("New")),
                Item::new(button("Open")),
                Item::new(button("Settings")),
            ])
            .max_width(220.0)
            .offset(8.0)
            .spacing(2.0),
        )])
        .safe_bounds_margin(0.0);

        let nav_bar = container(row![dropdown, horizontal()]);

        container(column![
            nav_bar,
            center(text("Nameless Music Player").size(28))
        ])
        .style(header())
        .into()
    }

    pub fn scale_factor(&self) -> f32 {
        match self {
            App::Loading => 1.0,
            App::Ready(state) => state.ui_scale,
        }
    }

    pub fn theme(&self) -> Theme {
        match self {
            App::Loading => Theme::default(),
            App::Ready(state) => state.theme.clone(),
        }
    }
}
