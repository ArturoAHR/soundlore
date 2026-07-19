use iced::{
    Point, Rectangle,
    advanced::{Layout, Shell, renderer},
    mouse::{self, Cursor},
    window,
};

use crate::ui::widgets::table::{
    Catalog, Table,
    mouse::TableArea,
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

        if let Some(position) = cursor.position() {
            state.mouse_interaction.area = self.get_position_table_area(state, bounds, position);
        }

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

        if state.is_pending_redraw_request() {
            shell.request_redraw_at(window::RedrawRequest::NextFrame);
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

        let click = state
            .mouse_interaction
            .press_mouse_button(button, state.previous_click);

        #[allow(clippy::single_match)]
        match button {
            mouse::Button::Left => {
                let Some(clicked_area) = state.mouse_interaction.area.as_ref() else {
                    return;
                };

                match clicked_area {
                    TableArea::Header {
                        column_id: Some(column_id),
                    } => {
                        self.handle_mouse_header_click(shell, column_id.clone());
                    }
                    TableArea::Body {
                        row_id: Some(row_id),
                    } => {
                        self.handle_mouse_row_click(state, shell, row_id.clone(), click.kind());
                    }
                    TableArea::Scroll {
                        scroll_area_offset: Some(scroll_area_offset),
                    } => {
                        self.handle_mouse_scroll_click(
                            state,
                            shell,
                            bounds,
                            cursor_position,
                            *scroll_area_offset,
                        );
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    pub fn handle_mouse_button_release(&self, state: &mut State, shell: &mut Shell<'_, Message>) {
        state.previous_click = state.mouse_interaction.release_mouse_button();

        if state.mouse_interaction.area.is_some() {
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
        if let Some(table_area) = state.mouse_interaction.area.as_ref()
            && state.mouse_interaction.click.is_some()
        {
            match table_area {
                TableArea::Body { row_id: Some(_) } => {
                    self.handle_mouse_row_drag(state, shell);
                }
                TableArea::Scroll {
                    scroll_area_offset: Some(scroll_area_offset),
                } => {
                    self.handle_mouse_scroll_drag(
                        state,
                        shell,
                        bounds,
                        cursor_position,
                        *scroll_area_offset,
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
                        *scroll_area_start_offset,
                        *scroll_area_end_offset,
                    );
                }
                _ => {}
            }
        }
    }
}
