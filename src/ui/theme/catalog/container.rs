use iced::{
    widget::container::{Catalog, Style},
    Border,
};

use crate::ui::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn pane<'a>() -> StyleFn<'a> {
    Box::new(|theme: &Theme| Style {
        background: Some(theme.palette.surface_raised.into()),
        text_color: Some(theme.palette.text),
        border: Border {
            color: theme.palette.border,
            width: theme.sizes.border.width,
            radius: theme.sizes.border.radius_md.into(),
        },
        ..Style::default()
    })
}

pub fn header<'a>() -> StyleFn<'a> {
    Box::new(|theme: &Theme| Style {
        background: Some(theme.palette.surface_raised.into()),
        text_color: Some(theme.palette.text_subtle),
        border: Border {
            color: theme.palette.border,
            width: theme.sizes.border.width,
            ..Border::default()
        },
        ..Style::default()
    })
}
