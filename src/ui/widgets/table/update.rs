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

use crate::ui::widgets::table::{Catalog, Table, state::Identifiable};

use crate::ui::widgets::table::state::State;

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
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
        let state = tree.state.downcast_mut::<State>();

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
