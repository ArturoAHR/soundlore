use iced::{Point, Rectangle, advanced::renderer};

use crate::ui::widgets::table::{
    Catalog, Table,
    bounds::{
        get_effective_scroll_area_bounds, get_table_body_bounds, get_table_grid_bounds,
        get_table_header_bounds, get_table_scroll_bounds,
    },
    mouse::TableArea,
    scroll::get_scroll_thumb_bounds,
    state::{Identifiable, State, TableIdentifier},
};

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn get_row_id_at_position(
        &self,
        bounds: Rectangle,
        position: Point,
    ) -> Option<TableIdentifier> {
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

                (row_start <= position.y && position.y <= row_end).then(|| record.id().clone())
            })
    }

    pub fn get_header_column_id_at_position(
        &self,
        bounds: Rectangle,
        position: Point,
    ) -> Option<TableIdentifier> {
        let column_offsets = self
            .column_offsets
            .iter()
            .map(|&column_offset| column_offset + bounds.x);

        self.columns
            .iter()
            .zip(column_offsets)
            .find_map(|(column, column_start)| {
                let column_end = column_start + column.width;

                (column_start <= position.x && position.x <= column_end).then(|| column.id.clone())
            })
    }

    pub fn get_position_table_area(
        &self,
        state: &State,
        bounds: Rectangle,
        position: Point,
    ) -> Option<TableArea> {
        if !bounds.contains(position) {
            return None;
        }

        let grid_bounds = get_table_grid_bounds(bounds, self.scroll_width);

        let body_bounds = get_table_body_bounds(grid_bounds, self.header_height);

        if body_bounds.contains(position) {
            let row_id = self.get_row_id_at_position(bounds, position);

            return Some(TableArea::Body { row_id });
        }

        let header_bounds = get_table_header_bounds(grid_bounds, self.header_height);

        if header_bounds.contains(position) {
            let column_id = self.get_header_column_id_at_position(bounds, position);

            return Some(TableArea::Header { column_id });
        }

        let scroll_bounds = get_table_scroll_bounds(bounds, self.scroll_width);
        let effective_scroll_area_bounds =
            get_effective_scroll_area_bounds(scroll_bounds, self.header_height);
        let scroll_thumb_bounds = get_scroll_thumb_bounds(
            effective_scroll_area_bounds,
            self.row_height * self.records.len() as f32,
            state.offset_y,
        );

        if let Some(scroll_thumb_bounds) = scroll_thumb_bounds
            && scroll_thumb_bounds.contains(position)
        {
            let scroll_area_start_offset = position.y - scroll_thumb_bounds.y;
            let scroll_area_end_offset = scroll_thumb_bounds.height - scroll_area_start_offset;

            return Some(TableArea::ScrollThumb {
                scroll_area_start_offset,
                scroll_area_end_offset,
            });
        }

        if scroll_bounds.contains(position) {
            return Some(TableArea::Scroll {
                scroll_area_offset: scroll_thumb_bounds.map_or_else(
                    || None,
                    |scroll_thumb_bounds| Some(scroll_thumb_bounds.height / 2.0),
                ),
            });
        }

        None
    }
}
