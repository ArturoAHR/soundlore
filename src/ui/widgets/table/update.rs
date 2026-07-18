use iced::{
    Event, Point, Rectangle,
    advanced::{
        Clipboard, Layout, Shell,
        mouse::{self, Cursor, click},
        renderer::{self},
        widget::Tree,
    },
    keyboard, window,
};

use crate::ui::widgets::table::{
    Catalog, Table,
    bounds::{get_effective_scroll_area_bounds, get_table_scroll_bounds},
    mouse::{TableArea, TableClick},
    select::{SelectOperation, select_values},
    state::{Identifiable, TableIdentifier},
};

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
        let keyboard_modifiers = state.keyboard_modifiers;
        let bounds = layout.bounds();

        match event {
            // Scrolling
            iced::Event::Mouse(event) => {
                let Some(cursor_position) = cursor.position() else {
                    return;
                };

                match event {
                    mouse::Event::WheelScrolled { delta } => {
                        // Cursor is outside of table
                        if !cursor.is_over(bounds) {
                            return;
                        }

                        // Mousepad scrolling is significantly faster than mouse scroll.
                        let delta_y = match delta {
                            mouse::ScrollDelta::Lines { x: _, y } => *y * 15.0,
                            mouse::ScrollDelta::Pixels { x: _, y } => *y,
                        };

                        state.offset_y += delta_y * self.row_height * -0.15;
                        state.offset_y = state.offset_y.clamp(
                            0.0,
                            (self.row_height * self.records.len() as f32
                                - (layout.bounds().height - self.header_height))
                                .max(0.0),
                        );

                        shell.request_redraw();
                        shell.capture_event();
                    }

                    // Selection / Clicking
                    mouse::Event::ButtonPressed(button) => {
                        // Cursor is outside of table
                        if !cursor.is_over(bounds) {
                            state.focus_state.widget = false;

                            return;
                        }

                        state.focus_state.widget = true;
                        state.focus_state.window = true;

                        match button {
                            mouse::Button::Left => {
                                let table_click = self.get_table_click(
                                    layout,
                                    state.offset_y,
                                    state.previous_click,
                                    cursor_position,
                                    mouse::Button::Left,
                                );

                                state.current_click = Some(table_click);

                                let Some(clicked_area) = table_click.table_area else {
                                    return;
                                };

                                match clicked_area {
                                    TableArea::Header => {
                                        let Some(clicked_column_id) = self
                                            .get_header_column_id_at_position(
                                                layout,
                                                cursor_position,
                                            )
                                        else {
                                            return;
                                        };

                                        if let Some(on_header_cell_click) =
                                            self.on_header_cell_click.as_ref()
                                        {
                                            shell.publish(on_header_cell_click(clicked_column_id));
                                            shell.capture_event();
                                        }
                                    }
                                    TableArea::Body => {
                                        let Some(clicked_row_id) =
                                            self.get_row_id_at_position(layout, cursor_position)
                                        else {
                                            return;
                                        };

                                        if let Some(on_row_select) = self.on_row_select.as_ref() {
                                            shell.capture_event();

                                            let row_ids = self.records.iter().map(Identifiable::id);

                                            let (selected_row_ids, anchor_row_id) = select_values(
                                                row_ids,
                                                self.selected_rows.iter(),
                                                SelectOperation::from_keyboard_modifiers(
                                                    keyboard_modifiers,
                                                    clicked_row_id,
                                                    state.selection_anchor_row_id.as_ref(),
                                                ),
                                            );

                                            state.selection_anchor_row_id = anchor_row_id;

                                            shell.publish(on_row_select(selected_row_ids));
                                        }

                                        if let Some(on_row_double_click) =
                                            self.on_row_double_click.as_ref()
                                            && matches!(
                                                table_click.click.kind(),
                                                click::Kind::Double
                                            )
                                        {
                                            shell.publish(on_row_double_click(
                                                clicked_row_id.to_owned(),
                                            ));
                                            shell.capture_event();
                                        }
                                    }
                                    TableArea::Scroll {
                                        scroll_area_offset: Some(scroll_area_offset),
                                    } => {
                                        state.offset_y = self.get_scroll_offset(
                                            layout,
                                            cursor_position.y,
                                            scroll_area_offset,
                                            scroll_area_offset,
                                        );

                                        shell.request_redraw();
                                        shell.capture_event();
                                    }
                                    _ => {}
                                }
                            }

                            _ => {}
                        }
                    }

                    mouse::Event::ButtonReleased(_button) => {
                        state.previous_click = state.current_click.take();

                        if state
                            .previous_click
                            .iter()
                            .any(|click| click.table_area.is_some())
                        {
                            shell.request_redraw();
                            shell.capture_event();
                        }
                    }

                    mouse::Event::CursorMoved { position } => {
                        if let Some(mut current_table_click) = state.current_click
                            && let Some(table_area) = current_table_click.table_area
                        {
                            current_table_click.current_position = *position;

                            match table_area {
                                TableArea::Body => {
                                    // TODO (v2): Add scroll on moving the mouse up or down the table body past a certain threshold
                                    if let Some(on_row_select) = self.on_row_select.as_ref() {
                                        let Some(current_position_row_id) = self
                                            .get_row_id_at_position(
                                                layout,
                                                current_table_click.current_position,
                                            )
                                        else {
                                            return;
                                        };

                                        let row_ids = self.records.iter().map(Identifiable::id);

                                        let mut select_operation =
                                            SelectOperation::from_keyboard_modifiers(
                                                keyboard_modifiers,
                                                current_position_row_id,
                                                state.selection_anchor_row_id.as_ref(),
                                            );

                                        if matches!(
                                            select_operation,
                                            SelectOperation::Single { .. }
                                                | SelectOperation::Toggle { .. }
                                        ) {
                                            select_operation = SelectOperation::Range {
                                                target_value: current_position_row_id,
                                                anchor_value: state
                                                    .selection_anchor_row_id
                                                    .as_ref(),
                                            }
                                        }

                                        let (selected_row_ids, anchor_row_id) = select_values(
                                            row_ids,
                                            self.selected_rows.iter(),
                                            select_operation,
                                        );

                                        state.selection_anchor_row_id = anchor_row_id;

                                        shell.publish(on_row_select(selected_row_ids));
                                    }
                                }
                                TableArea::Scroll {
                                    scroll_area_offset: Some(scroll_area_offset),
                                } => {
                                    state.offset_y = self.get_scroll_offset(
                                        layout,
                                        current_table_click.current_position.y,
                                        scroll_area_offset,
                                        scroll_area_offset,
                                    );

                                    shell.request_redraw_at(window::RedrawRequest::NextFrame);
                                    shell.capture_event();
                                }
                                TableArea::ScrollThumb {
                                    scroll_area_start_offset,
                                    scroll_area_end_offset,
                                } => {
                                    state.offset_y = self.get_scroll_offset(
                                        layout,
                                        current_table_click.current_position.y,
                                        scroll_area_start_offset,
                                        scroll_area_end_offset,
                                    );

                                    shell.request_redraw_at(window::RedrawRequest::NextFrame);
                                    shell.capture_event();
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if matches!(key.as_ref(), keyboard::Key::Character("a"))
                    && *modifiers == keyboard::Modifiers::COMMAND =>
            {
                if let Some(on_row_select) = self.on_row_select.as_ref()
                    && state.focus_state.is_focused()
                {
                    shell.capture_event();

                    let row_ids = self.records.iter().map(Identifiable::id);

                    let (selected_row_ids, anchor_row_id) = select_values(
                        row_ids,
                        self.selected_rows.iter(),
                        SelectOperation::All {
                            anchor_value: state.selection_anchor_row_id.as_ref(),
                        },
                    );

                    state.selection_anchor_row_id = anchor_row_id;

                    shell.publish(on_row_select(selected_row_ids));
                }
            }

            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = *modifiers;
            }
            Event::Window(window::Event::RedrawRequested(_))
                if (state.last_layout_offset_y - state.offset_y).abs() > 0.1 =>
            {
                shell.invalidate_layout();

                state.last_layout_offset_y = state.offset_y;
            }
            Event::Window(window::Event::Unfocused) => {
                state.focus_state.window = false;
            }
            Event::Window(window::Event::Focused) => state.focus_state.window = true,
            _ => {}
        }
    }

    fn get_row_id_at_position(
        &self,
        layout: Layout<'_>,
        position: Point<f32>,
    ) -> Option<&'a TableIdentifier> {
        let bounds = layout.bounds();

        let visible_records = &self.records[self.visible_row_range.clone()];
        let visible_row_offsets = self
            .row_offsets
            .iter()
            .map(|&row_offset| row_offset + bounds.y);

        visible_records
            .iter()
            .zip(visible_row_offsets)
            .find_map(|(record, row_start)| {
                let row_end = row_start + self.row_height;

                (row_start <= position.y && position.y <= row_end).then(|| record.id())
            })
    }

    fn get_header_column_id_at_position(
        &self,
        layout: Layout<'_>,
        position: Point<f32>,
    ) -> Option<TableIdentifier> {
        let bounds = layout.bounds();

        let column_offsets = self
            .column_offsets
            .iter()
            .map(|&column_offset| column_offset + bounds.x);

        self.columns
            .iter()
            .zip(column_offsets)
            .find_map(|(column, column_start)| {
                let column_end = column_start + column.width;

                (column_start <= position.x && position.x <= column_end)
                    .then_some(column.id.clone())
            })
    }

    fn get_table_click(
        &self,
        layout: Layout<'_>,
        scroll_offset: f32,
        previous_table_click: Option<TableClick>,
        cursor_position: Point,
        button: mouse::Button,
    ) -> TableClick {
        let bounds = layout.bounds();

        let table_area = TableArea::get_position_table_area(
            cursor_position,
            bounds,
            self.header_height,
            self.scroll_width,
            self.row_height * self.records.len() as f32,
            scroll_offset,
        );

        TableClick::new(cursor_position, button, table_area, previous_table_click)
    }

    fn get_scroll_offset(
        &self,
        layout: Layout<'_>,
        position: f32,
        scroll_area_start_offset: f32,
        scroll_area_end_offset: f32,
    ) -> f32
    where
        T: Identifiable,
        Message: 'a,
        Theme: Catalog,
        Renderer: renderer::Renderer,
    {
        let bounds = layout.bounds();

        let scroll_bounds = get_table_scroll_bounds(bounds, self.scroll_width);
        let effective_scroll_area_bounds =
            get_effective_scroll_area_bounds(scroll_bounds, self.header_height);
        let total_scrollable_content_length =
            self.row_height * self.records.len() as f32 - effective_scroll_area_bounds.height;

        let minimum_height = effective_scroll_area_bounds.y + scroll_area_start_offset;
        let maximum_height = (effective_scroll_area_bounds.y + effective_scroll_area_bounds.height
            - scroll_area_end_offset)
            .max(minimum_height);
        let position = position.clamp(minimum_height, maximum_height);

        let position_ratio =
            ((position - minimum_height) / (maximum_height - minimum_height)).clamp(0.0, 1.0);

        total_scrollable_content_length * position_ratio
    }
}
