use iced::{
    Alignment, Element, Length, Renderer,
    widget::{button, container, row, slider},
};

use crate::ui::{
    components::playback_bar::Message,
    theme::Theme,
    widgets::icons::{self, icon},
};

pub fn volume_bar<'a>(volume_percentage: u8, muted: bool) -> Element<'a, Message, Theme, Renderer> {
    let mut volume_percentage = volume_percentage.clamp(0, 100);

    let volume_icon = if muted || volume_percentage == 0 {
        volume_percentage = 0;
        icons::VOLUME_MUTED
    } else {
        icons::VOLUME
    };

    container(
        row![
            button(icon(volume_icon)).on_press(Message::MutePlayback),
            slider(0..=100, volume_percentage, Message::ChangeVolumePercentage,)
        ]
        .width(Length::Fixed(130.0))
        .align_y(Alignment::Center)
        .spacing(10.0),
    )
    .into()
}
