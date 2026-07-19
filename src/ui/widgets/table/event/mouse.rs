use iced::{
    Point, Rectangle,
    advanced::{Layout, Shell, mouse::Click, renderer},
    mouse::{self, Cursor},
    window,
};

use crate::ui::widgets::table::{
    Catalog, Table,
    state::{Identifiable, State},
};

use crate::ui::widgets::table::state::TableIdentifier;

#[derive(Debug, Clone, Default)]
pub struct MouseInteraction {
    pub area: Option<TableArea>,
    pub position: Point,
    pub click: Option<TableClick>,
}

impl MouseInteraction {
    pub fn press_mouse_button(
        &mut self,
        mouse_button: mouse::Button,
        previous_click: Option<TableClick>,
    ) -> Click {
        let click = Click::new(
            self.position,
            mouse_button,
            previous_click.map(|table_click| table_click.click),
        );

        self.click = Some(TableClick {
            area: self.area.clone(),
            click,
        });

        click
    }

    pub fn release_mouse_button(&mut self) -> Option<TableClick> {
        self.click.take()
    }
}

#[derive(Debug, Clone)]
pub struct TableClick {
    pub area: Option<TableArea>,
    pub click: Click,
}

impl TableClick {
    pub fn new(area: Option<TableArea>, click: Click) -> Self {
        Self { area, click }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableArea {
    Header {
        column_id: Option<TableIdentifier>,
    },
    Body {
        row_id: Option<TableIdentifier>,
    },
    Scroll {
        scroll_area_offset: Option<f32>,
    },
    ScrollThumb {
        scroll_area_start_offset: f32,
        scroll_area_end_offset: f32,
    },
}

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
            state.mouse_interaction.position = position;
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
            .press_mouse_button(button, state.previous_click.clone());

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
        if let Some(table_area) = state
            .mouse_interaction
            .click
            .as_ref()
            .and_then(|click| click.area.as_ref())
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
