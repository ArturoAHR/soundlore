use std::hash::Hash;

use iced::{
    Event, Rectangle,
    advanced::{
        Clipboard, Layout, Shell,
        mouse::Cursor,
        renderer::{self},
        widget::Tree,
    },
    window,
};

use crate::{
    traits::Identifiable,
    ui::widgets::table::{Catalog, Table, TableRow},
};

use crate::ui::widgets::table::state::State;

impl<'a, T, ColumnId, Message, Theme, Renderer> Table<'a, '_, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow + 'static,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone + 'static,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    #[allow(clippy::single_match)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_impl(
        &self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<T::Identifier, ColumnId>>();

        match event {
            iced::Event::Mouse(event) => {
                self.handle_mouse_event(state, layout, cursor, shell, *event);
            }

            Event::Keyboard(event) => {
                self.handle_keyboard_event(state, shell, event);
            }

            Event::Window(window::Event::RedrawRequested(current_time)) => {
                self.handle_window_redraw_request(state, shell, current_time);
            }
            Event::Window(window::Event::Unfocused) => {
                state.focus_state.window = false;
            }
            Event::Window(window::Event::Focused) => state.focus_state.window = true,
            _ => {}
        }
    }
}
