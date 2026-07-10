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
    widget: &mut Table<'a, T, Message, Theme, Renderer>,
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
    let limits = limits.width(limits.max().width - widget.scroll_width);
    let state = tree.state.downcast_mut::<State>();

    // Children Cell Generation

    let mut visible_row_range = get_visible_range(
        limits.max().height,
        widget.row_height,
        widget.header_height,
        state.offset_y,
    );

    visible_row_range.start = visible_row_range.start.clamp(0, widget.records.len());
    visible_row_range.end = visible_row_range.end.clamp(0, widget.records.len());

    widget.visible_row_range = visible_row_range;
    widget.body_cells = Vec::with_capacity(widget.visible_row_range.len() * widget.columns.len());

    for record in &widget.records[widget.visible_row_range.clone()] {
        for column in &widget.columns {
            widget.body_cells.push((column.view)(record).into());
        }
    }

    // Column Width Resolution

    let container_width = limits.max().width as f64;
    let column_widths = widget
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

    for (column, column_width) in widget.columns.iter_mut().zip(column_widths) {
        column.width = column_width as f32;
    }

    // Get Cell Offsets

    widget.column_offsets = Vec::with_capacity(widget.columns.len());
    let mut column_width_offset_sum = 0.0;
    for column in widget.columns.iter() {
        widget.column_offsets.push(column_width_offset_sum);
        column_width_offset_sum += column.width;
    }

    widget.row_offsets = Vec::with_capacity(widget.visible_row_range.clone().count());
    let mut row_height_offset_sum =
        widget.header_height - widget.row_height * (state.offset_y / widget.row_height).fract();
    for _ in widget.visible_row_range.clone() {
        widget.row_offsets.push(row_height_offset_sum);
        row_height_offset_sum += widget.row_height;
    }

    // Child Node Generation

    let mut nodes = Vec::new();

    if widget.has_header {
        for ((header_cell, column), column_offset) in widget
            .header_cells
            .iter_mut()
            .zip(&widget.columns)
            .zip(&widget.column_offsets)
        {
            let padding = column.header_padding.unwrap_or(widget.header_cell_padding);

            let limits = Limits::new(Size::ZERO, Size::new(column.width, widget.header_height));
            let mut tree = state.cell_state.get_mut_or_insert(
                HEADERS_ROW_IDENTIFIER,
                &column.id,
                &header_cell,
            );

            let mut node = positioned(
                &limits,
                column.width,
                widget.header_height,
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
    if widget.has_header {
        row_ids.insert(&header_row_id);
    }

    for (visible_row_number, (record_id, row_offset)) in widget.records
        [widget.visible_row_range.clone()]
    .iter()
    .map(|record| record.id())
    .zip(widget.row_offsets.iter())
    .enumerate()
    {
        let row_body_cell_range = visible_row_number * widget.columns.len()
            ..(visible_row_number + 1) * widget.columns.len();

        row_ids.insert(record_id);

        for (body_cell, (column, column_offset)) in widget.body_cells[row_body_cell_range]
            .iter_mut()
            .zip(widget.columns.iter().zip(&widget.column_offsets))
        {
            let padding = column.cell_padding.unwrap_or(widget.cell_padding);

            let limits = Limits::new(Size::ZERO, Size::new(column.width, widget.row_height));
            let mut tree = state
                .cell_state
                .get_mut_or_insert(record_id, &column.id, body_cell);

            let mut node = positioned(
                &limits,
                column.width,
                widget.row_height,
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
