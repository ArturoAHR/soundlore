use iced::{
    widget::{text, text::Catalog},
    Element, Font,
};

use crate::app::Message;

// music-player-icons.ttf
pub const LOADING: char = '\u{E830}';
pub const PLAY: char = '\u{E805}';
pub const STOP: char = '\u{E804}';
pub const PLAY_NEXT: char = '\u{E802}';
pub const PLAY_PREVIOUS: char = '\u{E803}';
pub const LOOP_TRACKLIST: char = '\u{E809}';
pub const SHUFFLE: char = '\u{E80A}';
pub const NO_SHUFFLE: char = '\u{E801}';
pub const EQUALIZER: char = '\u{E800}';
pub const MENU: char = '\u{E806}';

pub fn icon<'a, T>(codepoint: char) -> Element<'a, Message, T>
where
    T: Catalog + 'a,
{
    const ICON_FONT: Font = Font::with_name("music-player-icons");

    text(codepoint).font(ICON_FONT).into()
}
