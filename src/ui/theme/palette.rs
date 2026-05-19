use iced::Color;

use crate::ui::theme::color::rgb;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub surface: Color,
    pub surface_raised: Color,
    pub surface_overlay: Color,
    pub surface_sunken: Color,

    pub hover: Color,
    pub focus: Color,
    pub border: Color,

    pub text: Color,
    pub text_subtle: Color,
    pub text_muted: Color,

    pub accent: Color,
    pub accent_text: Color,

    pub in_progress: Color,
    pub danger: Color,

    pub tag_default: Color,
}

impl Palette {
    pub const DARK: Self = Self {
        surface: rgb(0x14, 0x14, 0x18),
        surface_raised: rgb(0x1c, 0x1c, 0x22),
        surface_overlay: rgb(0x24, 0x24, 0x2c),
        surface_sunken: rgb(0x0e, 0x0e, 0x12),
        hover: rgb(0x2a, 0x2a, 0x33),
        focus: rgb(0x6a, 0x90, 0xff),
        border: rgb(0x33, 0x33, 0x3d),
        text: rgb(0xea, 0xea, 0xea),
        text_subtle: rgb(0xb0, 0xb0, 0xb8),
        text_muted: rgb(0x70, 0x70, 0x78),
        accent: rgb(0x6a, 0x90, 0xff),
        accent_text: rgb(0x0a, 0x0a, 0x0a),
        in_progress: rgb(0xf2, 0xc2, 0x4a),
        danger: rgb(0xe0, 0x5a, 0x5a),
        tag_default: rgb(0x55, 0x60, 0x80),
    };

    pub const LIGHT: Self = Self {
        surface: rgb(0xf2, 0xf3, 0xf6),
        surface_raised: rgb(0xfb, 0xfb, 0xfd),
        surface_overlay: rgb(0xff, 0xff, 0xff),
        surface_sunken: rgb(0xe5, 0xe7, 0xec),
        focus: rgb(0x2f, 0x5a, 0xe0),
        hover: rgb(0xe1, 0xe4, 0xec),
        border: rgb(0xd2, 0xd5, 0xdd),
        text: rgb(0x16, 0x18, 0x1f),
        text_subtle: rgb(0x4a, 0x4d, 0x58),
        text_muted: rgb(0x82, 0x86, 0x92),
        accent: rgb(0x2f, 0x5a, 0xe0),
        accent_text: rgb(0xff, 0xff, 0xff),
        in_progress: rgb(0xc6, 0x85, 0x0c),
        danger: rgb(0xc6, 0x2f, 0x2f),
        tag_default: rgb(0x55, 0x60, 0x80),
    };
}
