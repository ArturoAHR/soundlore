use iced::{Background, Border, Color, Shadow, Vector};
use iced_aw::style::{
    menu_bar::{Catalog, Style},
    Status,
};

use crate::ui::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn primary(theme: &Theme, _status: Status) -> Style {
    let p = &theme.palette;
    let s = &theme.sizes;

    Style {
        bar_background: Background::Color(p.surface_raised),
        bar_border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        bar_shadow: Shadow::default(),

        menu_background: Background::Color(p.surface_overlay),
        menu_border: Border {
            color: p.border,
            width: s.border.width,
            radius: s.border.radius_md.into(),
        },
        menu_shadow: Shadow {
            color: Color {
                a: 0.35,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },

        path: Background::Color(p.hover),
        path_border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: s.border.radius_sm.into(),
        },
    }
}
