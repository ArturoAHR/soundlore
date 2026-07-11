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
    let grid_limits = limits.width(limits.max().width - widget.scroll_width);
    let state = tree.state.downcast_mut::<State>();

    // Children Cell Generation

    let mut visible_row_range = get_visible_range(
        grid_limits.max().height,
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

    let container_width = grid_limits.max().width as f64;
    let column_widths = widget
        .columns
        .iter()
        .map(|column| column.get_column_width())
        .collect();

    let column_widths = get_column_widths(container_width, column_widths);

    for (column, column_width) in widget.columns.iter_mut().zip(column_widths) {
        column.width = column_width as f32;
    }

    // Get Cell Offsets

    widget.column_offsets = widget
        .columns
        .iter()
        .scan(0.0, |offset_accumulator, column| {
            let current_offset = *offset_accumulator;

            *offset_accumulator += column.width;

            Some(current_offset)
        })
        .collect();

    let row_offset_start =
        widget.header_height - widget.row_height * (state.offset_y / widget.row_height).fract();
    widget.row_offsets = (0..widget.visible_row_range.len())
        .map(|visible_row_number| row_offset_start + widget.row_height * visible_row_number as f32)
        .collect();

    // Child Node Generation

    let mut nodes = Vec::new();

    if widget.has_header {
        for (header_cell, column, column_offset) in izip!(
            widget.header_cells.iter_mut(),
            &widget.columns,
            &widget.column_offsets
        ) {
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

    let visible_row_record_ids = widget.records[widget.visible_row_range.clone()]
        .iter()
        .map(|record| record.id());
    let row_body_cell_groups = widget.body_cells.chunks_mut(widget.columns.len());

    for (record_id, row_offset, row_body_cells) in izip!(
        visible_row_record_ids,
        &widget.row_offsets,
        row_body_cell_groups
    ) {
        row_ids.insert(record_id);

        for (body_cell, column, column_offset) in izip!(
            row_body_cells.iter_mut(),
            &widget.columns,
            &widget.column_offsets
        ) {
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
