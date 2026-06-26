use iced::{Element, Renderer, widget::button};

use crate::ui::{
    components::navigation_bar::Message,
    theme::Theme,
    widgets::{
        icons::{self, icon},
        menu::{
            dropdown_menu, dropdown_menu_grouping_option, dropdown_menu_option, dropdown_toggle,
        },
    },
};

pub fn navigation_bar_menu<'a>(theme: &Theme) -> Element<'a, Message, Theme, Renderer> {
    let dropdown = dropdown_toggle(
        theme,
        button(icon(icons::MENU)),
        dropdown_menu(
            theme,
            vec![
                dropdown_menu_grouping_option(
                    theme,
                    "File",
                    dropdown_menu(
                        theme,
                        vec![
                            dropdown_menu_option(theme, "Add new files to library", None),
                            dropdown_menu_option(
                                theme,
                                "Scan folder for new files",
                                Some(Message::SelectedScanDirectoryOption),
                            ),
                        ],
                    ),
                ),
                dropdown_menu_option(theme, "Edit", None),
                dropdown_menu_option(theme, "View", None),
                dropdown_menu_option(theme, "Controls", None),
                dropdown_menu_option(theme, "Help", None),
            ],
        ),
    );

    dropdown.into()
}
