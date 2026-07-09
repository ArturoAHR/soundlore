use std::collections::HashSet;

use iced::{
    Point, Size,
    advanced::{
        layout::{Limits, Node, positioned},
        renderer::{self},
        widget::Tree,
    },
};

use crate::ui::{
    utils::table::{
        column::{ColumnWidth, get_column_widths},
        virtualization::get_visible_range,
    },
    widgets::table::{
        Catalog, Table,
        state::{HEADERS_ROW_IDENTIFIER, Identifiable},
    },
};

use crate::ui::widgets::table::state::State;

/// Creates the table cells with virtualization and the layout for the table, if the header
/// is present, first column count child nodes are the header cell nodes.
pub fn layout<'a, T, Message, Theme, Renderer>(
    table: &mut Table<'a, T, Message, Theme, Renderer>,
    tree: &mut Tree,
    renderer: &Renderer,
    limits: &Limits,
) -> Node
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    let limits = limits.width(limits.max().width - table.scroll_width);
    let state = tree.state.downcast_mut::<State>();

    // Children Cell Generation

    let mut visible_row_range = get_visible_range(
        limits.max().height,
        table.row_height,
        table.header_height,
        state.offset_y,
    );

    visible_row_range.start = visible_row_range.start.clamp(0, table.records.len());
    visible_row_range.end = visible_row_range.end.clamp(0, table.records.len());

    table.visible_row_range = visible_row_range;
    table.body_cells = Vec::with_capacity(table.visible_row_range.len() * table.columns.len());

    for record in &table.records[table.visible_row_range.clone()] {
        for column in &table.columns {
            table.body_cells.push((column.view)(record).into());
        }
    }

    // Column Width Resolution

    let container_width = limits.max().width as f64;
    let column_widths = table
        .columns
        .iter()
        .map(|column| {
            if column.resizable {
                ColumnWidth::Resizable {
                    width: column.width as f64,
                    min_width: column.min_width as f64,
                }
            } else {
                ColumnWidth::Fixed {
                    width: column.width as f64,
                }
            }
        })
        .collect();

    let column_widths = get_column_widths(container_width, column_widths);

    for (column, column_width) in table.columns.iter_mut().zip(column_widths) {
        column.width = column_width as f32;
    }

    // Get Cell Offsets

    table.column_offsets = Vec::with_capacity(table.columns.len());
    let mut column_width_offset_sum = 0.0;
    for column in table.columns.iter() {
        table.column_offsets.push(column_width_offset_sum);
        column_width_offset_sum += column.width;
    }

    table.row_offsets = Vec::with_capacity(table.visible_row_range.clone().count());
    let mut row_height_offset_sum =
        table.header_height - table.row_height * (state.offset_y / table.row_height).fract();
    for _ in table.visible_row_range.clone() {
        table.row_offsets.push(row_height_offset_sum);
        row_height_offset_sum += table.row_height;
    }

    // Child Node Generation

    let mut nodes = Vec::new();

    if table.has_header {
        for ((header_cell, column), column_offset) in table
            .header_cells
            .iter_mut()
            .zip(&table.columns)
            .zip(&table.column_offsets)
        {
            let padding = column.header_padding.unwrap_or(table.header_cell_padding);

            let limits = Limits::new(Size::ZERO, Size::new(column.width, table.header_height));
            let mut tree = state.cell_state.get_mut_or_insert(
                HEADERS_ROW_IDENTIFIER,
                &column.id,
                &header_cell,
            );

            let mut node = positioned(
                &limits,
                column.width,
                table.header_height,
                padding,
                |limits| {
                    header_cell
                        .as_widget_mut()
                        .layout(&mut tree, renderer, &limits.loose())
                },
                |node, size| node.align(column.align_x.into(), column.align_y.into(), size),
            );

            node = node.move_to(Point::new(*column_offset, 0.0));

            nodes.push(node);
        }
    }

    let mut row_ids: HashSet<&String> = HashSet::new();

    let header_row_id = HEADERS_ROW_IDENTIFIER.to_owned();
    if table.has_header {
        row_ids.insert(&header_row_id);
    }

    for (visible_row_number, (record_id, row_offset)) in table.records
        [table.visible_row_range.clone()]
    .iter()
    .map(|record| record.id())
    .zip(table.row_offsets.iter())
    .enumerate()
    {
        let row_body_cell_range = visible_row_number * table.columns.len()
            ..(visible_row_number + 1) * table.columns.len();

        row_ids.insert(record_id);

        for (body_cell, (column, column_offset)) in table.body_cells[row_body_cell_range]
            .iter_mut()
            .zip(table.columns.iter().zip(&table.column_offsets))
        {
            let padding = column.cell_padding.unwrap_or(table.cell_padding);

            let limits = Limits::new(Size::ZERO, Size::new(column.width, table.row_height));
            let mut tree = state
                .cell_state
                .get_mut_or_insert(record_id, &column.id, body_cell);

            let mut node = positioned(
                &limits,
                column.width,
                table.row_height,
                padding,
                |limits| {
                    body_cell
                        .as_widget_mut()
                        .layout(&mut tree, renderer, &limits.loose())
                },
                |node, size| node.align(column.align_x.into(), column.align_y.into(), size),
            );

            node = node.move_to(Point::new(*column_offset, *row_offset));

            nodes.push(node);
        }
    }

    state.cell_state.prune(row_ids);

    Node::with_children(limits.max(), nodes)
}
