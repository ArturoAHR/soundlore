use iced::{Point, Rectangle, advanced::renderer};

use crate::ui::widgets::table::{
    Catalog, Table,
    state::{Identifiable, TableIdentifier},
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
        position: Point<f32>,
    ) -> Option<&'a TableIdentifier> {
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

    pub fn get_header_column_id_at_position(
        &self,
        bounds: Rectangle,
        position: Point<f32>,
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

                (column_start <= position.x && position.x <= column_end)
                    .then_some(column.id.clone())
            })
    }
}
