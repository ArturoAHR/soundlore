use iced::{
    Rectangle,
    advanced::{
        Layout,
        mouse::Cursor,
        renderer::{self, Quad},
        widget::Tree,
    },
};

use crate::ui::widgets::table::{
    BodyRowStatus, Catalog, CellStatus, CellType, ScrollState, ScrollStatus, Table,
    state::{HEADERS_ROW_IDENTIFIER, Identifiable},
};

use crate::ui::widgets::table::state::State;

pub fn draw<'a, T, Message, Theme, Renderer>(
    widget: &Table<'a, T, Message, Theme, Renderer>,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    _style: &renderer::Style,
    layout: Layout<'_>,
    cursor: Cursor,
    viewport: &Rectangle,
) where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    let state = tree.state.downcast_ref::<State>();
    let bounds = layout.bounds();

    let grid_bounds = Rectangle {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width - widget.scroll_width,
        height: bounds.height,
    };

    let table_style = theme.table_style(&widget.table_class);

    // Render background
    renderer.fill_quad(
        Quad {
            bounds: layout.bounds(),
            border: table_style.border,
            ..Default::default()
        },
        table_style.background,
    );

    // Body

    let mut body_cell_layouts = layout.children().skip(widget.columns.len());

    let body_bounds = Rectangle {
        x: grid_bounds.x,
        y: grid_bounds.y + widget.header_height,
        width: grid_bounds.width,
        height: grid_bounds.height - widget.header_height,
    };

    // Clipping body cells to table body bounds
    renderer.with_layer(body_bounds, |renderer| {
        // Render table body rows background
        for (row_number, row_offset) in widget.visible_row_range.clone().zip(&widget.row_offsets) {
            let row_bounds = Rectangle {
                x: body_bounds.x,
                y: body_bounds.y + row_offset - widget.header_height,
                width: body_bounds.width,
                height: widget.row_height,
            };

            let mut row_status = BodyRowStatus::Default;
            if cursor.is_over(row_bounds) {
                row_status = BodyRowStatus::Hovered
            }

            let row_style = theme.body_row_style(&widget.body_row_class, row_status, row_number);

            renderer.fill_quad(
                Quad {
                    bounds: row_bounds,
                    ..Default::default()
                },
                row_style.background,
            );
        }

        for (visible_row_number, (row_id, row_offset)) in widget.records
            [widget.visible_row_range.clone()]
        .iter()
        .map(|record| record.id())
        .zip(&widget.row_offsets)
        .enumerate()
        {
            let row_bounds = Rectangle {
                x: body_bounds.x,
                y: body_bounds.y + row_offset - widget.header_height,
                width: body_bounds.width,
                height: widget.row_height,
            };

            let row_body_cell_range = visible_row_number * widget.columns.len()
                ..(visible_row_number + 1) * widget.columns.len();

            for ((cell, cell_layout), column_id) in widget.body_cells[row_body_cell_range]
                .iter()
                .zip(body_cell_layouts.by_ref().take(widget.columns.len()))
                .zip(widget.columns.iter().map(|column| &column.id))
            {
                // Cell bounds need to be intersected with table body bounds in case the cells
                // belong to the bottom-most row which can be cut in half.
                let Some(cell_bounds) = cell_layout.bounds().intersection(&body_bounds) else {
                    continue;
                };

                let mut cell_status = CellStatus::Default;
                if cursor.is_over(row_bounds) {
                    cell_status = CellStatus::Hovered
                }

                let cell_style = theme.cell_style(&widget.cell_class, cell_status, CellType::Body);

                // Clipping cell contents to cell bounds
                renderer.with_layer(cell_bounds, |renderer| {
                    if let Some(cell_state) = state.cell_state.get(row_id, column_id) {
                        cell.as_widget().draw(
                            cell_state,
                            renderer,
                            theme,
                            &renderer::Style {
                                text_color: cell_style.text_color,
                            },
                            // Gets inner layout since it's a padding wrapper for the child
                            cell_layout.child(0),
                            cursor,
                            viewport,
                        );
                    }
                });
            }
        }
    });

    // Header

    let header_cell_layouts = layout.children().take(widget.columns.len());

    if widget.has_header {
        let header_bounds = Rectangle {
            x: grid_bounds.x,
            y: grid_bounds.y,
            width: grid_bounds.width,
            height: widget.header_height,
        };

        // Clipping body cells to table header bounds
        renderer.with_layer(header_bounds, |renderer| {
            // render header background
            renderer.fill_quad(
                Quad {
                    bounds: header_bounds,
                    ..Default::default()
                },
                table_style.header_background,
            );

            for (((cell, cell_layout), column_id), column_offset) in widget
                .header_cells
                .iter()
                .zip(header_cell_layouts)
                .zip(widget.columns.iter().map(|column| &column.id))
                .zip(&widget.column_offsets)
            {
                // Cell bounds need to be intersected with table header bounds in case the current
                // cell is the right most column header one.
                let Some(cell_bounds) = cell_layout.bounds().intersection(&header_bounds) else {
                    continue;
                };

                let mut cell_status = CellStatus::Default;
                if cursor.is_over(cell_bounds) {
                    cell_status = CellStatus::Hovered;
                }

                let cell_style =
                    theme.cell_style(&widget.cell_class, cell_status, CellType::Header);

                // Clipping cell contents to cell bounds
                renderer.with_layer(cell_bounds, |renderer| {
                    if let Some(cell_state) =
                        state.cell_state.get(HEADERS_ROW_IDENTIFIER, column_id)
                    {
                        cell.as_widget().draw(
                            cell_state,
                            renderer,
                            theme,
                            &renderer::Style {
                                text_color: cell_style.text_color,
                            },
                            // Gets inner layout since it's a padding wrapper for the child
                            cell_layout.child(0),
                            cursor,
                            viewport,
                        );
                    }
                });

                // Render column header divisory line
                if *column_offset > 0.0 {
                    let header_column_separator_bounds = Rectangle {
                        x: header_bounds.x + *column_offset - 1.0,
                        y: header_bounds.y,
                        width: 1.0,
                        height: header_bounds.height,
                    };

                    renderer.fill_quad(
                        Quad {
                            bounds: header_column_separator_bounds,
                            ..Default::default()
                        },
                        table_style.header_separator_x,
                    );
                }
            }

            let header_body_separator_bounds = Rectangle {
                x: header_bounds.x,
                y: header_bounds.y + header_bounds.height - 1.0,
                width: header_bounds.width,
                height: 1.0,
            };

            renderer.fill_quad(
                Quad {
                    bounds: header_body_separator_bounds,
                    ..Default::default()
                },
                table_style.header_body_separator,
            );
        });
    }

    // Scrollbar

    let scroll_bounds = Rectangle {
        x: bounds.x + grid_bounds.width,
        y: bounds.y,
        width: widget.scroll_width,
        height: bounds.height,
    };

    renderer.with_layer(scroll_bounds, |renderer| {
        let effective_scroll_area_height = scroll_bounds.height - widget.header_height;
        let scroll_thumb_height = (effective_scroll_area_height
            * (body_bounds.height / (widget.row_height * widget.records.len() as f32)))
            .max(40.0);
        let scroll_thumb_offset = (effective_scroll_area_height - scroll_thumb_height)
            * (state.offset_y
                / (widget.row_height * widget.records.len() as f32 - body_bounds.height));
        let scroll_thumb_horizontal_padding = widget.scroll_width * 0.25;
        let scroll_thumb_width = widget.scroll_width - scroll_thumb_horizontal_padding * 2.0;

        let scroll_thumb_bounds = Rectangle {
            x: scroll_bounds.x + scroll_thumb_horizontal_padding,
            y: scroll_bounds.y + widget.header_height + scroll_thumb_offset,
            width: scroll_thumb_width,
            height: scroll_thumb_height,
        };

        let mut scroll_state = ScrollState {
            vertical_scroll_status: ScrollStatus::Default,
        };

        if cursor.is_over(scroll_thumb_bounds) {
            scroll_state.vertical_scroll_status = ScrollStatus::Hovered;
        }

        let scroll_style = theme.scroll_style(&widget.scroll_class, scroll_state);

        renderer.fill_quad(
            Quad {
                bounds: scroll_bounds,
                ..Default::default()
            },
            scroll_style.vertical_scroll.background,
        );

        renderer.fill_quad(
            Quad {
                bounds: scroll_thumb_bounds,
                border: scroll_style.vertical_scroll.thumb_border,
                ..Default::default()
            },
            scroll_style.vertical_scroll.thumb_background,
        );
    });
}
