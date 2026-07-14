use iced::{
    Event, Point, Rectangle,
    advanced::{
        Clipboard, Layout, Shell,
        mouse::{self, Click, Cursor, click},
        renderer::{self},
        widget::Tree,
    },
};

use crate::ui::widgets::table::{
    Catalog, Table,
    state::Identifiable,
    widget::select::{SelectOperation, select_values},
};

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
    let keyboard_modifiers = state.keyboard_modifiers;
    let bounds = layout.bounds();

    match event {
        // Scrolling
        iced::Event::Mouse(event) => {
            let Some(cursor_position) = cursor.position() else {
                return;
            };

            match event {
                iced::mouse::Event::WheelScrolled { delta } => {
                    // Cursor is outside of table
                    if !cursor.is_over(bounds) {
                        return;
                    }

                    // Mousepad scrolling is significantly faster than mouse scroll.
                    let delta_y = match delta {
                        iced::mouse::ScrollDelta::Lines { x: _, y } => *y * 15.0,
                        iced::mouse::ScrollDelta::Pixels { x: _, y } => *y,
                    };

                    state.offset_y += delta_y * widget.row_height * -0.15;
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
                    // Cursor is outside of table
                    if !cursor.is_over(bounds) {
                        return;
                    }

                    let table_click = get_table_click(
                        widget,
                        layout,
                        state.previous_click,
                        cursor_position,
                        mouse::Button::Left,
                    );

                    state.previous_click = Some(table_click);

                    let Some(clicked_area) = table_click.clicked_area else {
                        return;
                    };

                    match clicked_area {
                        TableArea::Header => {
                            let Some(clicked_column_id) =
                                widget.columns.iter().zip(&widget.column_offsets).find_map(
                                    |(column, &column_start)| {
                                        let column_end = column_start + column.width;

                                        (column_start <= cursor_position.x
                                            && cursor_position.x <= column_end)
                                            .then_some(&column.id)
                                    },
                                )
                            else {
                                return;
                            };

                            if let Some(on_header_cell_click) = widget.on_header_cell_click.as_ref()
                            {
                                shell.publish(on_header_cell_click(clicked_column_id.to_owned()));
                                shell.capture_event();
                            }
                        }
                        TableArea::Body => {
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
                                shell.capture_event();

                                let row_ids = widget.records.iter().map(Identifiable::id);
                                let default_anchor_row_id = String::new();
                                let anchor_row_id = state
                                    .selection_anchor_row_id
                                    .as_ref()
                                    .or_else(|| widget.records.first().map(Identifiable::id))
                                    .unwrap_or(&default_anchor_row_id);

                                let (selected_row_ids, anchor_row_id) = select_values(
                                    row_ids,
                                    &widget.selected_rows,
                                    clicked_row_id,
                                    anchor_row_id,
                                    SelectOperation::from_keyboard_modifiers(keyboard_modifiers),
                                );

                                state.selection_anchor_row_id = Some(anchor_row_id);

                                shell.publish(on_row_select(selected_row_ids));
                            }

                            if let Some(on_row_double_click) = widget.on_row_double_click.as_ref()
                                && matches!(table_click.click.kind(), click::Kind::Double)
                            {
                                shell.publish(on_row_double_click(clicked_row_id.to_owned()));
                                shell.capture_event();
                            }
                        }
                        TableArea::Scroll => {}
                    }
                }
                _ => {}
            }
        }
        Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
            state.keyboard_modifiers = *modifiers;
        }
        _ => {}
    }
}

fn get_table_click<'a, T, Message, Theme, Renderer>(
    widget: &Table<'a, T, Message, Theme, Renderer>,
    layout: Layout<'_>,
    previous_click: Option<TableClick>,
    cursor_position: Point,
    button: mouse::Button,
) -> TableClick
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    let bounds = layout.bounds();

    let click = Click::new(
        cursor_position,
        button,
        previous_click.map(|table_click| table_click.click),
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
    for (bounds, table_area) in [body_bounds, header_bounds, scroll_bounds].iter().zip([
        TableArea::Body,
        TableArea::Header,
        TableArea::Scroll,
    ]) {
        if bounds.contains(cursor_position) {
            clicked_area = Some(table_area);
        }
    }

    TableClick {
        clicked_area,
        click,
    }
}
