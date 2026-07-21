use std::{hash::Hash, time::Instant};

use iced::advanced::{Shell, renderer};

use crate::{
    traits::Identifiable,
    ui::widgets::table::{Catalog, Table, TableRow, state::State},
};

impl<'a, T, ColumnId, Message, Theme, Renderer> Table<'a, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn handle_window_redraw_request(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
        shell: &mut Shell<'_, Message>,
        _current_time: &Instant,
    ) {
        if state.take_pending_layout_invalidation() {
            shell.invalidate_layout();
        }
    }
}
