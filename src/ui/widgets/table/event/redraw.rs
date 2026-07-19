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
    pub fn handle_window_redraw_request(
        &self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        _current_time: &Instant,
    ) {
        if state.take_pending_layout_invalidation() {
            shell.invalidate_layout();
        }
    }
}
