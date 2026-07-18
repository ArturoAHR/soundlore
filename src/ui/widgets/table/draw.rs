use iced::{
    Rectangle,
    advanced::{
        Layout,
        mouse::Cursor,
        renderer::{self, Quad},
        widget::Tree,
    },
};
use itertools::izip;

use crate::ui::widgets::table::{
    BodyRowStatus, Catalog, CellStatus, CellType, ScrollState, ScrollStatus, Table, TableStyle,
    bounds::{
        get_effective_scroll_area_bounds, get_table_body_bounds, get_table_body_row_bounds,
        get_table_grid_bounds, get_table_header_bounds, get_table_scroll_bounds,
    },
    scroll::get_scroll_thumb_bounds,
    state::{HEADERS_ROW_IDENTIFIER, Identifiable},
};

use crate::ui::widgets::table::state::State;

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_impl(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let grid_bounds = get_table_grid_bounds(bounds, self.scroll_width);

        let table_style = theme.table_style(&self.class);

        self.draw_table_background(renderer, style, &table_style, layout);

        self.draw_table_body(
            state,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
            grid_bounds,
        );

        self.draw_table_header(
            state,
            renderer,
            theme,
            style,
            &table_style,
            layout,
            cursor,
            viewport,
            grid_bounds,
        );

        self.draw_table_scroll(state, renderer, theme, style, cursor, bounds);
    }

    fn draw_table_background(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        table_style: &TableStyle,
        layout: Layout<'_>,
    ) {
        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                border: table_style.border,
                ..Default::default()
            },
            table_style.background,
        );
    }

    fn draw_table_body(
        &self,
        state: &State,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        grid_bounds: Rectangle,
    ) {
        let mut body_cell_layouts = layout.children().skip(self.columns.len());

        let body_bounds = get_table_body_bounds(grid_bounds, self.header_height);

        // Clipping body cells to table body bounds
        renderer.with_layer(body_bounds, |renderer| {
            let visible_row_ids = self.records[self.visible_row_range.clone()]
                .iter()
                .map(Identifiable::id);

            // Render table body rows background
            for (row_number, row_id, row_offset) in izip!(
                self.visible_row_range.clone(),
                visible_row_ids.clone(),
                &self.row_offsets
            ) {
                let row_bounds = get_table_body_row_bounds(
                    body_bounds,
                    self.header_height,
                    self.row_height,
                    *row_offset,
                );

                let row_status = if self.selected_rows.contains(row_id) {
                    BodyRowStatus::Selected
                } else if cursor.is_over(row_bounds) {
                    BodyRowStatus::Hovered
                } else {
                    BodyRowStatus::Default
                };

                let row_style = theme.body_row_style(&self.body_row_class, row_status, row_number);

                renderer.fill_quad(
                    Quad {
                        bounds: row_bounds,
                        ..Default::default()
                    },
                    row_style.background,
                );
            }

            for (visible_row_number, row_id, row_offset) in
                izip!(0.., visible_row_ids, &self.row_offsets)
            {
                let row_bounds = get_table_body_row_bounds(
                    body_bounds,
                    self.header_height,
                    self.row_height,
                    *row_offset,
                );

                let row_body_cell_range = visible_row_number * self.columns.len()
                    ..(visible_row_number + 1) * self.columns.len();

                for ((cell, cell_layout), column_id) in self.body_cells[row_body_cell_range]
                    .iter()
                    .zip(body_cell_layouts.by_ref().take(self.columns.len()))
                    .zip(self.columns.iter().map(|column| &column.id))
                {
                    // Cell bounds need to be intersected with table body bounds in case the cells
                    // belong to the bottom-most row which can be cut in half.
                    let Some(cell_bounds) = cell_layout.bounds().intersection(&body_bounds) else {
                        continue;
                    };

                    let cell_status = if self.selected_rows.contains(row_id) {
                        CellStatus::Selected
                    } else if cursor.is_over(row_bounds) {
                        CellStatus::Hovered
                    } else {
                        CellStatus::Default
                    };

                    let cell_style =
                        theme.cell_style(&self.cell_class, cell_status, CellType::Body);

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
    }

    fn draw_table_header(
        &self,
        state: &State,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        table_style: &TableStyle,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        grid_bounds: Rectangle,
    ) {
        let header_cell_layouts = layout.children().take(self.columns.len());

        if self.has_header {
            let header_bounds = get_table_header_bounds(grid_bounds, self.header_height);

            // Clipping header cells to table header bounds
            renderer.with_layer(header_bounds, |renderer| {
                // render header background
                renderer.fill_quad(
                    Quad {
                        bounds: header_bounds,
                        ..Default::default()
                    },
                    table_style.header_background,
                );

                let column_ids = self.columns.iter().map(|column| &column.id);

                for (cell, cell_layout, column_id, column_offset) in izip!(
                    &self.header_cells,
                    header_cell_layouts,
                    column_ids,
                    &self.column_offsets
                ) {
                    // Cell bounds need to be intersected with table header bounds in case the current
                    // cell is the right most column header one.
                    let Some(cell_bounds) = cell_layout.bounds().intersection(&header_bounds)
                    else {
                        continue;
                    };

                    let cell_status = if cursor.is_over(cell_bounds) {
                        CellStatus::Hovered
                    } else {
                        CellStatus::Default
                    };

                    let cell_style =
                        theme.cell_style(&self.cell_class, cell_status, CellType::Header);

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
    }

    fn draw_table_scroll(
        &self,
        state: &State,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        cursor: Cursor,
        bounds: Rectangle,
    ) {
        let scroll_bounds = get_table_scroll_bounds(bounds, self.scroll_width);

        renderer.with_layer(scroll_bounds, |renderer| {
            let effective_scroll_area_bounds =
                get_effective_scroll_area_bounds(scroll_bounds, self.header_height);

            let scroll_thumb_bounds = get_scroll_thumb_bounds(
                effective_scroll_area_bounds,
                self.row_height * self.records.len() as f32,
                state.offset_y,
            );

            let mut scroll_state = ScrollState {
                vertical_scroll_status: ScrollStatus::Default,
            };

            if let Some(scroll_thumb_bounds) = scroll_thumb_bounds
                && cursor.is_over(scroll_thumb_bounds)
            {
                scroll_state.vertical_scroll_status = ScrollStatus::Hovered;
            }

            let scroll_style = theme.scroll_style(&self.scroll_class, scroll_state);

            renderer.fill_quad(
                Quad {
                    bounds: scroll_bounds,
                    ..Default::default()
                },
                scroll_style.vertical_scroll.background,
            );

            if let Some(scroll_thumb_bounds) = scroll_thumb_bounds {
                renderer.fill_quad(
                    Quad {
                        bounds: scroll_thumb_bounds,
                        border: scroll_style.vertical_scroll.thumb_border,
                        ..Default::default()
                    },
                    scroll_style.vertical_scroll.thumb_background,
                );
            }
        });
    }
}
