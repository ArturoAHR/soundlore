use std::time::Instant;

use iced::advanced::{Shell, renderer};

use crate::ui::widgets::table::{
    Catalog, Table,
    state::{Identifiable, State},
};

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub(in super::super) fn handle_window_redraw_request(
        &self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        _current_time: &Instant,
    ) {
        if (state.last_layout_offset_y - state.offset_y).abs() > 0.1 {
            shell.invalidate_layout();

            state.last_layout_offset_y = state.offset_y;
        }
    }
}
