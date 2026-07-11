use iced::{
    Event, Point, Rectangle,
    advanced::{
        Clipboard, Layout, Shell,
        mouse::{self, Click, Cursor, click},
        renderer::{self},
        widget::Tree,
    },
};

use crate::ui::widgets::table::{Catalog, Table, state::Identifiable};

use crate::ui::widgets::table::state::State;

#[derive(Debug, Clone, Copy)]
pub struct TableClick {
    pub clicked_area: Option<TableArea>,
    pub click: Click,
}

#[derive(Debug, Clone, Copy)]
pub enum TableArea {
    Header,
    Body,
    Scroll,
}

#[allow(clippy::single_match)]
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
        iced::Event::Mouse(event) => {
            // Cursor is outside of table
            if !cursor.is_over(bounds) {
                match event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if let Some(on_row_select) = widget.on_row_select.as_ref() {
                            shell.publish(on_row_select(vec![]));
                        }
                    }
                    _ => {}
                }
                return;
            }

            let Some(cursor_position) = cursor.position() else {
                return;
            };

            match event {
                iced::mouse::Event::WheelScrolled { delta } => {
                    let delta_y = match delta {
                        iced::mouse::ScrollDelta::Lines { x: _, y }
                        | iced::mouse::ScrollDelta::Pixels { x: _, y } => *y,
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
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let table_click =
                        get_table_click(widget, layout, tree, cursor_position, mouse::Button::Left);

                    if matches!(table_click.clicked_area, Some(TableArea::Body)) {
                        let visible_records = &widget.records[widget.visible_row_range.clone()];
                        let visible_row_offsets = widget
                            .row_offsets
                            .iter()
                            .map(|&row_offset| row_offset + bounds.y);

                        let Some(clicked_row_id) = visible_records
                            .iter()
                            .zip(visible_row_offsets)
                            .find_map(|(record, row_start)| {
                                let row_end = row_start + widget.row_height;

                                (row_start <= cursor_position.y && cursor_position.y <= row_end)
                                    .then(|| record.id())
                            })
                        else {
                            return;
                        };

                        if let Some(on_row_select) = widget.on_row_select.as_ref() {
                            shell.publish(on_row_select(vec![clicked_row_id.to_owned()]));
                            shell.capture_event();
                        }

                        if let Some(on_row_double_click) = widget.on_row_double_click.as_ref()
                            && matches!(table_click.click.kind(), click::Kind::Double)
                        {
                            shell.publish(on_row_double_click(clicked_row_id.to_owned()));
                            shell.capture_event();
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn get_table_click<'a, T, Message, Theme, Renderer>(
    widget: &Table<'a, T, Message, Theme, Renderer>,
    layout: Layout<'_>,
    tree: &mut Tree,
    cursor_position: Point,
    button: mouse::Button,
) -> TableClick
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    let state = tree.state.downcast_mut::<State>();
    let bounds = layout.bounds();

    let click = Click::new(
        cursor_position,
        button,
        state
            .previous_click
            .as_ref()
            .and_then(|table_click| Some(table_click.click)),
    );

    let grid_bounds = Rectangle {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width - widget.scroll_width,
        height: bounds.height,
    };

    let body_bounds = Rectangle {
        x: grid_bounds.x,
        y: grid_bounds.y + widget.header_height,
        width: grid_bounds.width,
        height: grid_bounds.height - widget.header_height,
    };

    let header_bounds = Rectangle {
        x: grid_bounds.x,
        y: grid_bounds.y,
        width: grid_bounds.width,
        height: widget.header_height,
    };

    let scroll_bounds = Rectangle {
        x: bounds.x + grid_bounds.width,
        y: bounds.y,
        width: widget.scroll_width,
        height: bounds.height,
    };

    let mut clicked_area = None;
    for (bounds, table_area) in vec![body_bounds, header_bounds, scroll_bounds]
        .iter()
        .zip(vec![TableArea::Body, TableArea::Header, TableArea::Scroll].into_iter())
    {
        if bounds.contains(cursor_position) {
            clicked_area = Some(table_area);
        }
    }

    let table_click = TableClick {
        clicked_area,
        click,
    };

    state.previous_click = Some(table_click);

    table_click
}
