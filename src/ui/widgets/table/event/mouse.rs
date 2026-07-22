use std::hash::Hash;

use iced::{
    Point, Rectangle,
    advanced::{Layout, Shell, mouse::Click, renderer},
    mouse::{self, Cursor},
    window,
};

use crate::{
    traits::Identifiable,
    ui::widgets::table::{Catalog, Table, TableRow, state::State},
};

#[derive(Debug, Clone)]
pub struct MouseInteraction<RowId, ColumnId> {
    pub area: Option<TableArea<RowId, ColumnId>>,
    pub position: Point,
    pub click: Option<TableClick<RowId, ColumnId>>,
}

impl<RowId, ColumnId> MouseInteraction<RowId, ColumnId>
where
    RowId: Clone,
    ColumnId: Clone,
{
    pub fn press_mouse_button(
        &mut self,
        mouse_button: mouse::Button,
        previous_click: Option<TableClick<RowId, ColumnId>>,
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

    pub fn release_mouse_button(&mut self) -> Option<TableClick<RowId, ColumnId>> {
        self.click.take()
    }
}

impl<RowId, ColumnId> Default for MouseInteraction<RowId, ColumnId> {
    fn default() -> Self {
        Self {
            area: None,
            click: None,
            position: Point::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableClick<RowId, ColumnId> {
    pub area: Option<TableArea<RowId, ColumnId>>,
    pub click: Click,
}

impl<RowId, ColumnId> TableClick<RowId, ColumnId> {
    pub fn new(area: Option<TableArea<RowId, ColumnId>>, click: Click) -> Self {
        Self { area, click }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableArea<RowId, ColumnId> {
    Header {
        column_id: Option<ColumnId>,
    },
    Body {
        row_id: Option<RowId>,
    },
    Scroll {
        scroll_area_offset: Option<f32>,
    },
    ScrollThumb {
        scroll_area_start_offset: f32,
        scroll_area_end_offset: f32,
    },
}

impl<'a, T, ColumnId, Message, Theme, Renderer> Table<'a, '_, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn handle_mouse_event(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
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

        if state.take_pending_redraw_request() {
            shell.request_redraw_at(window::RedrawRequest::NextFrame);
        }
    }

    pub fn handle_mouse_button_press(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
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

    pub fn handle_mouse_button_release(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
        shell: &mut Shell<'_, Message>,
    ) {
        state.previous_click = state.mouse_interaction.release_mouse_button();

        if state.mouse_interaction.area.is_some() {
            shell.request_redraw();
            shell.capture_event();
        }
    }

    pub fn handle_mouse_cursor_moved(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
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
