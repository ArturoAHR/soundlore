use std::hash::Hash;

use iced::{
    Point, Rectangle,
    advanced::{Shell, renderer},
    mouse::{self, Cursor},
    window,
};

use crate::{
    traits::Identifiable,
    ui::widgets::table::{
        Catalog, Table, TableRow,
        bounds::{get_effective_scroll_area_bounds, get_table_scroll_bounds},
        state::State,
    },
};

impl<'a, T, ColumnId, Message, Theme, Renderer> Table<'a, '_, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn handle_mouse_wheel_scroll(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
        bounds: Rectangle,
        cursor: Cursor,
        shell: &mut Shell<'_, Message>,
        delta: mouse::ScrollDelta,
    ) {
        // Cursor is outside of table
        if !cursor.is_over(bounds) {
            return;
        }

        // Mousepad scrolling is significantly faster than mouse scroll.
        let delta_y = match delta {
            mouse::ScrollDelta::Lines { x: _, y } => y * 15.0,
            mouse::ScrollDelta::Pixels { x: _, y } => y,
        };

        state.offset_y += delta_y * self.row_height * -0.15;
        state.offset_y = state.offset_y.clamp(
            0.0,
            (self.row_height * self.records.len() as f32 - (bounds.height - self.header_height))
                .max(0.0),
        );

        shell.request_redraw();
        shell.capture_event();
    }

    pub fn handle_mouse_scroll_click(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
        shell: &mut Shell<'_, Message>,
        bounds: Rectangle,
        cursor_position: Point,
        scroll_area_offset: f32,
    ) {
        state.offset_y = self.get_scroll_offset(
            bounds,
            cursor_position.y,
            scroll_area_offset,
            scroll_area_offset,
        );

        shell.request_redraw_at(window::RedrawRequest::NextFrame);
        shell.capture_event();
    }

    pub fn handle_mouse_scroll_drag(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
        shell: &mut Shell<'_, Message>,
        bounds: Rectangle,
        cursor_position: Point,
        scroll_area_offset: f32,
    ) {
        state.offset_y = self.get_scroll_offset(
            bounds,
            cursor_position.y,
            scroll_area_offset,
            scroll_area_offset,
        );

        shell.request_redraw_at(window::RedrawRequest::NextFrame);
        shell.capture_event();
    }

    pub fn handle_mouse_scroll_thumb_drag(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
        shell: &mut Shell<'_, Message>,
        bounds: Rectangle,
        cursor_position: Point,
        scroll_area_start_offset: f32,
        scroll_area_end_offset: f32,
    ) {
        state.offset_y = self.get_scroll_offset(
            bounds,
            cursor_position.y,
            scroll_area_start_offset,
            scroll_area_end_offset,
        );

        shell.request_redraw_at(window::RedrawRequest::NextFrame);
        shell.capture_event();
    }

    pub fn get_scroll_offset(
        &self,
        bounds: Rectangle,
        position: f32,
        scroll_area_start_offset: f32,
        scroll_area_end_offset: f32,
    ) -> f32 {
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
