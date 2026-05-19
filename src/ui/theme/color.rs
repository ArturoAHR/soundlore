use iced::Color;

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

    let [ar, ag, ab, aa] = a.into_linear();
    let [br, bg, bb, ba] = b.into_linear();

    Color::from_linear_rgba(
        ar + (br - ar) * t,
        ag + (bg - ag) * t,
        ab + (bb - ab) * t,
        aa + (ba - aa) * t,
    )
}

/// Makes the given color lighter by a given amount
pub fn lighten(c: Color, amount: f32) -> Color {
    mix(c, Color::WHITE, amount)
}

/// Makes the given color darker by a given amount
pub fn darken(c: Color, amount: f32) -> Color {
    mix(c, Color::BLACK, amount)
}
