use iced::border::Radius;
use iced_split::{Catalog, Style, StyleSheet};

use crate::ui::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme| Style {
            snap: false,
            focused: StyleSheet {
                color: theme.palette.border,
                radius: Radius::default(),
                width: 0.0,
            },
            unfocused: StyleSheet {
                color: theme.palette.border,
                radius: Radius::default(),
                width: 0.0,
            },
        })
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}
