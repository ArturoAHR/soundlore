use iced::{
    Border, Color,
    widget::slider::{Catalog, Handle, HandleShape, Rail, Status, Style},
};

use crate::ui::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| {
            let fill_color = match status {
                Status::Active => theme.palette.accent,
                Status::Hovered => theme.palette.focus,
                Status::Dragged => theme.palette.focus,
            };

            Style {
                rail: Rail {
                    backgrounds: (fill_color.into(), theme.palette.surface_sunken.into()),
                    width: theme.sizes.component.progress_thickness,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: (theme.sizes.component.progress_thickness / 2.0).into(),
                    },
                },
                handle: Handle {
                    shape: HandleShape::Circle {
                        radius: theme.sizes.component.progress_handle_diameter / 2.0,
                    },
                    background: fill_color.into(),
                    border_color: theme.palette.surface,
                    border_width: match status {
                        Status::Active => 0.0,
                        _ => theme.sizes.border.width,
                    },
                },
            }
        })
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}
