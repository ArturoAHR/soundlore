use std::path::PathBuf;

use rfd::AsyncFileDialog;
use sqlx::SqlitePool;

use iced::{
    widget::{button, center, column, container, row, space::horizontal, text},
    Element, Task,
};

use crate::{
    error::AppError,
    library::scanner::scan_files_in_directory,
    ui::{
        icons::{self, icon},
        theme::{catalog::container::header, Theme},
        widgets::menu::{
            dropdown_menu, dropdown_menu_grouping_option, dropdown_menu_option, dropdown_toggle,
        },
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
    ScanDirectory(Result<Vec<PathBuf>, AppError>),
    ScannedDirectory(Result<(), AppError>),
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
            Message::OpenDirectoryScanDialog => Task::perform(
                async {
                    AsyncFileDialog::new()
                        .pick_folders()
                        .await
                        .map(|handles| handles.iter().map(|handle| handle.path().into()).collect())
                        .ok_or(AppError::DialogCancelled)
                },
                Message::ScanDirectory,
            ),
            Message::ScanDirectory(Ok(directories)) => {
                let pool = self.pool.clone();

                Task::perform(
                    async move { scan_files_in_directory(&pool, directories).await },
                    Message::ScannedDirectory,
                )
            }
            Message::ScanDirectory(Err(_)) => Task::none(),
            Message::ScannedDirectory(_) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        let dropdown = dropdown_toggle(
            &self.theme,
            button(icon(icons::MENU)),
            dropdown_menu(
                &self.theme,
                vec![
                    dropdown_menu_grouping_option(
                        &self.theme,
                        "File",
                        dropdown_menu(
                            &self.theme,
                            vec![
                                dropdown_menu_option(&self.theme, "Add new files to library", None),
                                dropdown_menu_option(
                                    &self.theme,
                                    "Scan folder for new files",
                                    Some(Message::OpenDirectoryScanDialog),
                                ),
                            ],
                        ),
                    ),
                    dropdown_menu_option(&self.theme, "Edit", None),
                    dropdown_menu_option(&self.theme, "View", None),
                    dropdown_menu_option(&self.theme, "Controls", None),
                    dropdown_menu_option(&self.theme, "Help", None),
                ],
            ),
        );

        //     Menu::new(vec![
        //         Item::new(button("New")),
        //         Item::new(button("Open")),
        //         Item::new(button("Settings")),
        //     ])
        //     .max_width(220.0)
        //     .offset(8.0)
        //     .spacing(2.0),
        // )])
        // .safe_bounds_margin(self.theme.sizes.space.md);

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
