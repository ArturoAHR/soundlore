use iced::{
    Event, Rectangle,
    advanced::{
        Clipboard, Layout, Shell,
        mouse::{self, Click, Cursor, click},
        renderer::{self},
        widget::Tree,
    },
};

use crate::ui::widgets::table::{Catalog, Table, state::Identifiable};

use crate::ui::widgets::table::state::State;

pub fn update<'a, T, Message, Theme, Renderer>(
    widget: &mut Table<'a, T, Message, Theme, Renderer>,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    cursor: Cursor,
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
    let bounds = layout.bounds();

    match event {
        // Scrolling
        iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
            let delta_y = match delta {
                iced::mouse::ScrollDelta::Lines { x: _, y } => *y,
                iced::mouse::ScrollDelta::Pixels { x: _, y } => *y,
            };

            state.offset_y += delta_y * widget.row_height * -0.7;
            state.offset_y = state.offset_y.clamp(
                0.0,
                (widget.row_height * widget.records.len() as f32
                    - (layout.bounds().height - widget.header_height))
                    .max(0.0),
            );

            shell.invalidate_layout();
            shell.request_redraw();
            shell.capture_event();
        }

        // Selection / Clicking
        iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            let body_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + widget.header_height,
                width: bounds.width,
                height: bounds.height - widget.header_height,
            };

            let header_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: widget.header_height,
            };

            let scroll_bounds = Rectangle {
                x: bounds.x + bounds.width,
                y: bounds.y,
                width: widget.scroll_width,
                height: bounds.height,
            };

            if let Some(position) = cursor.position_over(body_bounds) {
                let mut clicked_row_id = None;

                for (row_id, row_offset) in widget.records[widget.visible_row_range.clone()]
                    .iter()
                    .map(|record| record.id())
                    .zip(&widget.row_offsets)
                {
                    let row_bounds = Rectangle {
                        x: body_bounds.x,
                        y: body_bounds.y + row_offset - widget.header_height,
                        width: body_bounds.width,
                        height: widget.row_height,
                    };

                    if cursor.is_over(row_bounds) {
                        clicked_row_id = Some(row_id);
                    }
                }

                if let Some(clicked_row_id) = clicked_row_id {
                    let new_click = Click::new(position, mouse::Button::Left, state.last_click);

                    state.last_click = Some(new_click);

                    if let Some(on_row_select) = widget.on_row_select.as_ref() {
                        shell.publish(on_row_select(vec![clicked_row_id.to_owned()]));
                        shell.capture_event();
                    }

                    if let Some(on_row_double_click) = widget.on_row_double_click.as_ref()
                        && matches!(new_click.kind(), click::Kind::Double)
                    {
                        shell.publish(on_row_double_click(clicked_row_id.to_owned()));
                        shell.capture_event();
                    }
                }
            }
        }
        _ => {}
    }
}
