use iced::Color;
use palette::{Clamp, Darken, IntoColor, Lighten, Mix, Oklab, Oklch, Srgb};

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}

/// Reassigns alpha value of a Color
pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

/// Mixes the first colors passed in with the second by a given amount.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);

    let alpha = a.a + (b.a - a.a) * t;

    let oklab_a = convert_color_to_oklab(a);
    let oklab_b = convert_color_to_oklab(b);

    convert_srgb_to_color(oklab_a.mix(oklab_b, t), alpha)
}

/// Makes the given color lighter by a given amount
pub fn lighten(c: Color, amount: f32) -> Color {
    let oklch = convert_color_to_oklch(c);

    convert_srgb_to_color(oklch.lighten(amount), c.a)
}

/// Makes the given color darker by a given amount
pub fn darken(c: Color, amount: f32) -> Color {
    let oklch = convert_color_to_oklch(c);

    convert_srgb_to_color(oklch.darken(amount), c.a)
}

fn convert_color_to_oklch(c: Color) -> Oklch {
    Srgb::new(c.r, c.g, c.b).into_color()
}

fn convert_color_to_oklab(c: Color) -> Oklab {
    Srgb::new(c.r, c.g, c.b).into_color()
}

fn convert_srgb_to_color(srgb: impl IntoColor<Srgb>, a: f32) -> Color {
    let rgb = srgb.into_color().clamp();

    Color {
        r: rgb.red,
        g: rgb.green,
        b: rgb.blue,
        a,
    }
}
