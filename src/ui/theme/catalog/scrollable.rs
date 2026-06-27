use iced::{
    Background, Border, Color, Shadow,
    border::Radius,
    widget::{
        container,
        scrollable::{AutoScroll, Catalog, Rail, Scroller, Status, Style},
    },
};

use crate::ui::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| Style {
            auto_scroll: AutoScroll {
                background: theme.palette.surface_sunken.into(),
                border: Border {
                    color: theme.palette.border,
                    width: theme.sizes.border.width,
                    radius: Radius::default(),
                },
                icon: theme.palette.surface,
                shadow: Shadow::default(),
            },
            container: container::Style::default(),
            gap: None,
            horizontal_rail: Rail {
                background: Some(theme.palette.surface_sunken.into()),
                border: Border {
                    color: theme.palette.border,
                    width: theme.sizes.border.width,
                    radius: Radius::default(),
                },
                scroller: Scroller {
                    background: theme.palette.surface_raised.into(),
                    border: Border {
                        color: theme.palette.border,
                        width: theme.sizes.border.width,
                        radius: Radius::default(),
                    },
                },
            },
            vertical_rail: Rail {
                background: Some(theme.palette.surface_sunken.into()),
                border: Border {
                    color: theme.palette.border,
                    width: theme.sizes.border.width,
                    radius: Radius::default(),
                },
                scroller: Scroller {
                    background: theme.palette.surface_raised.into(),
                    border: Border {
                        color: theme.palette.border,
                        width: theme.sizes.border.width,
                        radius: Radius::default(),
                    },
                },
            },
        })
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}
