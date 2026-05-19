use std::borrow::Cow;

use crate::ui::theme::{palette::Palette, sizes::Sizes};

pub mod catalog;
pub mod color;
pub mod palette;
pub mod sizes;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: Cow<'static, str>,
    pub mode: Mode,
    pub palette: Palette,
    pub sizes: Sizes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

impl Theme {
    pub const DARK: Self = Self {
        name: Cow::Borrowed("Dark"),
        mode: Mode::Dark,
        palette: Palette::DARK,
        sizes: Sizes::DEFAULT,
    };

    pub const LIGHT: Self = Self {
        name: Cow::Borrowed("Light"),
        mode: Mode::Light,
        palette: Palette::LIGHT,
        sizes: Sizes::DEFAULT,
    };

    pub const BUILTIN_THEMES: &'static [&'static Self] = &[&Self::DARK, &Self::LIGHT];

    pub fn by_name(name: &str) -> Self {
        Self::BUILTIN_THEMES
            .iter()
            .copied()
            .find(|theme| theme.name == name)
            .cloned()
            .unwrap_or_else(|| Self::DARK.clone())
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::DARK
    }
}
