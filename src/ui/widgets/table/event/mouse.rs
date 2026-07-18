use iced::{
    Point, Rectangle,
    advanced::{Layout, Shell, renderer},
    mouse::{self, Cursor},
};

use crate::ui::widgets::table::{
    Catalog, Table,
    mouse::{TableArea, TableClick},
    state::{Identifiable, State},
};

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn handle_mouse_event(
        &self,
        state: &mut State,
        layout: Layout<'_>,
        cursor: Cursor,
        shell: &mut Shell<'_, Message>,
        event: mouse::Event,
    ) {
        let bounds = layout.bounds();

        match event {
            mouse::Event::WheelScrolled { delta } => {
                self.handle_mouse_wheel_scroll(state, bounds, cursor, shell, delta);
            }

            mouse::Event::ButtonPressed(button) => {
                self.handle_mouse_button_press(state, bounds, cursor, shell, button);
            }

            mouse::Event::ButtonReleased(_button) => {
                self.handle_mouse_button_release(state, shell);
            }

            mouse::Event::CursorMoved { position } => {
                self.handle_mouse_cursor_moved(state, shell, bounds, position);
            }
            _ => {}
        }
    }

    pub fn handle_mouse_button_press(
        &self,
        state: &mut State,
        bounds: Rectangle,
        cursor: Cursor,
        shell: &mut Shell<'_, Message>,
        button: mouse::Button,
    ) {
        // Cursor is outside of table
        let Some(cursor_position) = cursor.position_over(bounds) else {
            state.focus_state.widget = false;

            return;
        };

        state.focus_state.widget = true;
        state.focus_state.window = true;

        match button {
            mouse::Button::Left => {
                let table_click = self.get_table_click(
                    bounds,
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
                        self.handle_mouse_header_click(shell, bounds, cursor_position);
                    }
                    TableArea::Body => {
                        self.handle_mouse_row_click(
                            state,
                            shell,
                            bounds,
                            cursor_position,
                            table_click.click.kind(),
                        );
                    }
                    TableArea::Scroll {
                        scroll_area_offset: Some(scroll_area_offset),
                    } => {
                        self.handle_mouse_scroll_click(
                            state,
                            shell,
                            bounds,
                            cursor_position,
                            scroll_area_offset,
                        );
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    pub fn handle_mouse_button_release(&self, state: &mut State, shell: &mut Shell<'_, Message>) {
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

    pub fn handle_mouse_cursor_moved(
        &self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        bounds: Rectangle,
        cursor_position: Point,
    ) {
        if let Some(mut current_table_click) = state.current_click
            && let Some(table_area) = current_table_click.table_area
        {
            current_table_click.current_position = cursor_position;
            state.current_click = Some(current_table_click);

            match table_area {
                TableArea::Body => {
                    // TODO (v2): Add scroll on moving the mouse up or down the table body past a certain threshold
                    self.handle_mouse_row_drag(state, shell, bounds, cursor_position);
                }
                TableArea::Scroll {
                    scroll_area_offset: Some(scroll_area_offset),
                } => {
                    self.handle_mouse_scroll_drag(
                        state,
                        shell,
                        bounds,
                        cursor_position,
                        scroll_area_offset,
                    );
                }
                TableArea::ScrollThumb {
                    scroll_area_start_offset,
                    scroll_area_end_offset,
                } => {
                    self.handle_mouse_scroll_thumb_drag(
                        state,
                        shell,
                        bounds,
                        cursor_position,
                        scroll_area_start_offset,
                        scroll_area_end_offset,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn get_table_click(
        &self,
        bounds: Rectangle,
        scroll_offset: f32,
        previous_table_click: Option<TableClick>,
        cursor_position: Point,
        button: mouse::Button,
    ) -> TableClick {
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
}
