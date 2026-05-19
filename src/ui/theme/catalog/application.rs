use iced::theme::{
    self, Mode as IcedThemeMode, Palette as IcedThemePalette, Style as IcedThemeStyle,
};

use crate::ui::theme::{Mode, Theme};

impl theme::Base for Theme {
    fn default(preference: IcedThemeMode) -> Self {
        match preference {
            theme::Mode::Light => Theme::LIGHT,
            theme::Mode::Dark => Theme::DARK,
            theme::Mode::None => Theme::DARK,
        }
    }

    fn mode(&self) -> IcedThemeMode {
        match self.mode {
            Mode::Light => IcedThemeMode::Light,
            Mode::Dark => IcedThemeMode::Dark,
        }
    }

    fn base(&self) -> IcedThemeStyle {
        IcedThemeStyle {
            background_color: self.palette.surface,
            text_color: self.palette.text,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn palette(&self) -> Option<IcedThemePalette> {
        Some(IcedThemePalette {
            background: self.palette.surface,
            text: self.palette.text,
            primary: self.palette.accent,
            success: self.palette.accent,
            warning: self.palette.in_progress,
            danger: self.palette.danger,
        })
    }
}
