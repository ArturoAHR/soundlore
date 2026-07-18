use std::collections::HashSet;

use iced::{
    Point, Size,
    advanced::{
        layout::{Limits, Node, positioned},
        renderer::{self},
        widget::Tree,
    },
};
use itertools::izip;

use crate::ui::{
    utils::table::{column::get_column_widths, virtualization::get_visible_range},
    widgets::table::{
        Catalog, Column, Table,
        state::{HEADERS_ROW_IDENTIFIER, Identifiable},
    },
};

use crate::ui::widgets::table::state::State;

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates the table cells with virtualization and the layout for the table, if the header
    /// is present, first column count child nodes are the header cell nodes.
    pub(super) fn layout_impl(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        let grid_limits = limits.width(limits.max().width - self.scroll_width);
        let state = tree.state.downcast_mut::<State>();

        // Children Cell Generation

        let mut visible_row_range = get_visible_range(
            grid_limits.max().height,
            self.row_height,
            self.header_height,
            state.offset_y,
        );

        visible_row_range.start = visible_row_range.start.clamp(0, self.records.len());
        visible_row_range.end = visible_row_range.end.clamp(0, self.records.len());

        self.visible_row_range = visible_row_range;
        self.body_cells = Vec::with_capacity(self.visible_row_range.len() * self.columns.len());

        for record in &self.records[self.visible_row_range.clone()] {
            for column in &self.columns {
                self.body_cells.push((column.view)(record));
            }
        }

        // Column Width Resolution

        let container_width = From::<f32>::from(grid_limits.max().width);
        let column_widths = self.columns.iter().map(Column::get_column_width).collect();

        let column_widths = get_column_widths(container_width, column_widths);

        for (column, column_width) in self.columns.iter_mut().zip(column_widths) {
            column.width = column_width as f32;
        }

        // Get Cell Offsets

        self.column_offsets = self
            .columns
            .iter()
            .scan(0.0, |offset_accumulator, column| {
                let current_offset = *offset_accumulator;

                *offset_accumulator += column.width;

                Some(current_offset)
            })
            .collect();

        // Determines where the rows start by subtracting from the header height the length
        // the first row that is below overlapping it.
        let row_offset_start =
            self.header_height - self.row_height * (state.offset_y / self.row_height).fract();
        self.row_offsets = (0..self.visible_row_range.len())
            .map(|visible_row_number| {
                row_offset_start + self.row_height * visible_row_number as f32
            })
            .collect();

        // Child Node Generation

        let mut nodes = Vec::new();

        if self.has_header {
            for (header_cell, column, column_offset) in izip!(
                self.header_cells.iter_mut(),
                &self.columns,
                &self.column_offsets
            ) {
                let padding = column.header_padding.unwrap_or(self.header_cell_padding);

                let limits = Limits::new(Size::ZERO, Size::new(column.width, self.header_height));
                let tree = state.cell_state.get_mut_or_insert(
                    HEADERS_ROW_IDENTIFIER,
                    &column.id,
                    header_cell,
                );

                let mut node = positioned(
                    &limits,
                    column.width,
                    self.header_height,
                    padding,
                    |limits| {
                        header_cell
                            .as_widget_mut()
                            .layout(tree, renderer, &limits.loose())
                    },
                    |node, size| node.align(column.align_x.into(), column.align_y.into(), size),
                );

                node = node.move_to(Point::new(*column_offset, 0.0));

                nodes.push(node);
            }
        }

        let mut row_ids: HashSet<&String> = HashSet::new();

        let header_row_id = HEADERS_ROW_IDENTIFIER.to_owned();
        if self.has_header {
            row_ids.insert(&header_row_id);
        }

        let visible_row_record_ids = self.records[self.visible_row_range.clone()]
            .iter()
            .map(Identifiable::id);
        let row_body_cell_groups = self.body_cells.chunks_mut(self.columns.len());

        for (record_id, row_offset, row_body_cells) in izip!(
            visible_row_record_ids,
            &self.row_offsets,
            row_body_cell_groups
        ) {
            row_ids.insert(record_id);

            for (body_cell, column, column_offset) in izip!(
                row_body_cells.iter_mut(),
                &self.columns,
                &self.column_offsets
            ) {
                let padding = column.cell_padding.unwrap_or(self.cell_padding);

                let limits = Limits::new(Size::ZERO, Size::new(column.width, self.row_height));
                let tree = state
                    .cell_state
                    .get_mut_or_insert(record_id, &column.id, body_cell);

                let mut node = positioned(
                    &limits,
                    column.width,
                    self.row_height,
                    padding,
                    |limits| {
                        body_cell
                            .as_widget_mut()
                            .layout(tree, renderer, &limits.loose())
                    },
                    |node, size| node.align(column.align_x.into(), column.align_y.into(), size),
                );

                node = node.move_to(Point::new(*column_offset, *row_offset));

                nodes.push(node);
            }
        }

        state.cell_state.prune(&row_ids);

        Node::with_children(limits.max(), nodes)
    }
}
