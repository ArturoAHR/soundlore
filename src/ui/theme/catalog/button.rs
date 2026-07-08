use iced::{
    Border,
    widget::button::{Catalog, Status, Style},
};

use crate::ui::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| {
            let background = match status {
                Status::Active => theme.palette.surface_raised,
                Status::Pressed => theme.palette.surface_overlay,
                Status::Disabled => theme.palette.surface_sunken,
                Status::Hovered => theme.palette.hover,
            };

            Style {
                background: Some(background.into()),
                text_color: theme.palette.text,
                border: Border {
                    color: theme.palette.border,
                    width: theme.sizes.border.width,
                    radius: theme.sizes.border.radius_sm.into(),
                },
                ..Style::default()
            }
        })
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}
