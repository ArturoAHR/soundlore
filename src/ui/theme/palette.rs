use iced::Color;

use crate::ui::theme::color::rgb;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub surface: Color,
    pub surface_raised: Color,
    pub surface_overlay: Color,
    pub surface_sunken: Color,

    pub accent: Color,

    pub hover: Color,
    pub focus: Color,
    pub selected: Color,

    pub border: Color,

    pub text: Color,
    pub text_subtle: Color,
    pub text_muted: Color,
    pub text_selected: Color,
    pub text_accent: Color,

    pub in_progress: Color,
    pub danger: Color,

    pub tag_default: Color,
}

impl Palette {
    pub const DARK: Self = Self {
        surface: rgb(0x1a, 0x1a, 0x1a),
        surface_raised: rgb(0x16, 0x16, 0x16),
        surface_overlay: rgb(0x1e, 0x1e, 0x1e),
        surface_sunken: rgb(0x11, 0x11, 0x11),
        accent: rgb(0xf8, 0x7d, 0x20),
        hover: rgb(0x22, 0x22, 0x22),
        focus: rgb(0xf8, 0x7d, 0x20),
        selected: rgb(0x22, 0x1d, 0x12),
        border: rgb(0x2a, 0x2a, 0x2a),
        text: rgb(0xd4, 0xd4, 0xd4),
        text_subtle: rgb(0x88, 0x88, 0x88),
        text_muted: rgb(0x55, 0x55, 0x55),
        text_accent: rgb(0x11, 0x11, 0x11),
        text_selected: rgb(0xe8, 0xa0, 0x60),
        in_progress: rgb(0xf8, 0x7d, 0x20),
        danger: rgb(0xc0, 0x39, 0x2b),
        tag_default: rgb(0x2c, 0x2c, 0x2c),
    };

    pub const LIGHT: Self = Self {
        surface: rgb(0xf2, 0xf2, 0xf2),
        surface_raised: rgb(0xfa, 0xfa, 0xfa),
        surface_overlay: rgb(0xff, 0xff, 0xff),
        surface_sunken: rgb(0xe8, 0xe8, 0xe8),
        accent: rgb(0xd0, 0x62, 0x10),
        hover: rgb(0xe2, 0xe2, 0xe2),
        focus: rgb(0xd0, 0x62, 0x10),
        selected: rgb(0xf5, 0xe8, 0xd8),
        border: rgb(0xd0, 0xd0, 0xd0),
        text: rgb(0x18, 0x18, 0x18),
        text_subtle: rgb(0x55, 0x55, 0x55),
        text_muted: rgb(0x99, 0x99, 0x99),
        text_selected: rgb(0x8a, 0x45, 0x08),
        text_accent: rgb(0xff, 0xff, 0xff),
        in_progress: rgb(0xd0, 0x62, 0x10),
        danger: rgb(0xb0, 0x2a, 0x20),
        tag_default: rgb(0xd8, 0xd8, 0xd8),
    };
}
