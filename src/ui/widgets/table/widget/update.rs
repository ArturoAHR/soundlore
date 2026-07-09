use iced::{
    Event, Rectangle,
    advanced::{
        Clipboard, Layout, Shell,
        mouse::Cursor,
        renderer::{self},
        widget::Tree,
    },
};

use crate::ui::widgets::table::{Catalog, Table, state::Identifiable};

use crate::ui::widgets::table::state::State;

pub fn update<'a, T, Message, Theme, Renderer>(
    table: &mut Table<'a, T, Message, Theme, Renderer>,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    _cursor: Cursor,
    _renderer: &Renderer,
    _clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    _viewport: &Rectangle,
) where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    let state = tree.state.downcast_mut::<State>();

    match event {
        iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
            let delta_y = match delta {
                iced::mouse::ScrollDelta::Lines { x: _, y } => *y,
                iced::mouse::ScrollDelta::Pixels { x: _, y } => *y,
            };

            state.offset_y += delta_y * table.row_height * -0.7;
            state.offset_y = state.offset_y.clamp(
                0.0,
                (table.row_height * table.records.len() as f32
                    - (layout.bounds().height - table.header_height))
                    .max(0.0),
            );

            shell.invalidate_layout();
            shell.request_redraw();
            shell.capture_event();
        }
        _ => {}
    }
}
