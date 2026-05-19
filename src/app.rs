use std::path::PathBuf;

use sqlx::SqlitePool;

use iced::{
    widget::{button, center, column, container, row, space::horizontal, text},
    Element, Task,
};

use iced_aw::menu::{Item, Menu, MenuBar};

use crate::{
    error::AppError,
    ui::{
        icons::{self, icon},
        theme::{catalog::container::header, Theme},
    },
};

#[derive(Debug)]
pub struct App {
    pub pool: SqlitePool,
    pub ui_scale: f32,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenDirectoryScanDialog,
    ScanDirectory(Result<PathBuf, AppError>),
}

impl App {
    pub fn new(pool: SqlitePool, theme: Theme, ui_scale: f32) -> (Self, Task<Message>) {
        (
            App {
                pool,
                theme,
                ui_scale,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        String::from("Nameless Music Player")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenDirectoryScanDialog => Task::none(),
            Message::ScanDirectory(_) => Task::none(),
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
        .safe_bounds_margin(self.theme.sizes.space.md);

        let nav_bar = container(row![dropdown, horizontal()]);

        container(column![
            nav_bar,
            center(text("Nameless Music Player").size(28))
        ])
        .style(header())
        .into()
    }

    pub fn scale_factor(&self) -> f32 {
        self.ui_scale
    }

    pub fn theme(&self) -> Theme {
        self.theme.to_owned()
    }
}
