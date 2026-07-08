use std::collections::HashSet;

use iced::{
    Border, Color, Event, Length, Point, Rectangle, Size,
    advanced::{
        Clipboard, Layout, Shell,
        layout::{Limits, Node},
        mouse::Cursor,
        renderer::{self, Quad},
        widget::{Tree, Widget, tree},
    },
};

use crate::ui::{
    utils::table::{
        column::{ColumnWidth, get_column_widths},
        virtualization::get_visible_range,
    },
    widgets::table::{
        BodyRowStatus, Catalog, CellStatus, CellType, Table,
        state::{HEADERS_ROW_IDENTIFIER, Identifiable},
    },
};

use crate::ui::widgets::table::state::State;

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    /// Creates the table cells with virtualization and the layout for the table, if the header
    /// is present, first column count child nodes are the header cell nodes.
    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_mut::<State>();

        // Children Cell Generation

        let mut visible_row_range = get_visible_range(
            limits.max().height,
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
                self.body_cells.push((column.view)(record).into());
            }
        }

        // Column Width Resolution

        let container_width = limits.max().width as f64;
        let column_widths = self
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

        for (column, column_width) in self.columns.iter_mut().zip(column_widths) {
            column.width = column_width as f32;
        }

        // Get Cell Offsets

        self.column_offsets = Vec::with_capacity(self.columns.len());
        let mut column_width_offset_sum = 0.0;
        for column in self.columns.iter() {
            self.column_offsets.push(column_width_offset_sum);
            column_width_offset_sum += column.width;
        }

        self.row_offsets = Vec::with_capacity(self.visible_row_range.clone().count());
        let mut row_height_offset_sum =
            self.header_height - self.row_height * (state.offset_y / self.row_height).fract();
        for _ in self.visible_row_range.clone() {
            self.row_offsets.push(row_height_offset_sum);
            row_height_offset_sum += self.row_height;
        }

        // Child Node Generation

        let mut nodes = Vec::new();

        if self.has_header {
            for ((header_cell, column), column_offset) in self
                .header_cells
                .iter_mut()
                .zip(&self.columns)
                .zip(&self.column_offsets)
            {
                let limits = Limits::new(Size::ZERO, Size::new(column.width, self.header_height));
                let mut tree = state.cell_state.get_mut_or_insert(
                    HEADERS_ROW_IDENTIFIER,
                    &column.id,
                    &header_cell,
                );

                let mut node = header_cell
                    .as_widget_mut()
                    .layout(&mut tree, renderer, &limits);

                node = node.move_to(Point::new(*column_offset, 0.0)).align(
                    column.align_x.into(),
                    column.align_y.into(),
                    Size::new(column.width, self.header_height),
                );

                nodes.push(node);
            }
        }

        let mut row_ids: HashSet<&String> = HashSet::new();

        let header_row_id = HEADERS_ROW_IDENTIFIER.to_owned();
        if self.has_header {
            row_ids.insert(&header_row_id);
        }

        for (visible_row_number, (record_id, row_offset)) in self.records
            [self.visible_row_range.clone()]
        .iter()
        .map(|record| record.id())
        .zip(self.row_offsets.iter())
        .enumerate()
        {
            let row_body_cell_range = visible_row_number * self.columns.len()
                ..(visible_row_number + 1) * self.columns.len();

            row_ids.insert(record_id);

            for (body_cell, (column, column_offset)) in self.body_cells[row_body_cell_range]
                .iter_mut()
                .zip(self.columns.iter().zip(&self.column_offsets))
            {
                let limits = Limits::new(Size::ZERO, Size::new(column.width, self.row_height));
                let mut tree = state
                    .cell_state
                    .get_mut_or_insert(record_id, &column.id, body_cell);

                let mut node = body_cell
                    .as_widget_mut()
                    .layout(&mut tree, renderer, &limits);

                node = node.move_to(Point::new(*column_offset, *row_offset)).align(
                    column.align_x.into(),
                    column.align_y.into(),
                    Size::new(column.width, self.row_height),
                );

                nodes.push(node);
            }
        }

        state.cell_state.prune(row_ids);

        Node::with_children(limits.max(), nodes)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        // Background drawing

        let table_style = theme.table_style(&self.table_class);

        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                border: table_style.border,
                ..Default::default()
            },
            Color::WHITE,
        );

        let body_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y + self.header_height,
            width: bounds.width,
            height: bounds.height - self.header_height,
        };

        let header_cell_layouts = layout.children().take(self.columns.len());
        let mut body_cell_layouts = layout.children().skip(self.columns.len());

        // Clipping body cells to table body bounds
        renderer.with_layer(body_bounds, |renderer| {
            // Render table body rows background
            for (row_number, row_offset) in self.visible_row_range.clone().zip(&self.row_offsets) {
                let row_bounds = Rectangle {
                    x: body_bounds.x,
                    y: body_bounds.y + row_offset - self.header_height,
                    width: body_bounds.width,
                    height: self.row_height,
                };

                let mut row_status = BodyRowStatus::Default;
                if cursor.is_over(row_bounds) {
                    row_status = BodyRowStatus::Hovered
                }

                let row_style = theme.body_row_style(&self.body_row_class, row_status, row_number);

                renderer.fill_quad(
                    Quad {
                        bounds: row_bounds,
                        border: Border {
                            color: Color::TRANSPARENT,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    row_style.background,
                );
            }

            for (visible_row_number, (row_id, row_offset)) in self.records
                [self.visible_row_range.clone()]
            .iter()
            .map(|record| record.id())
            .zip(&self.row_offsets)
            .enumerate()
            {
                let row_bounds = Rectangle {
                    x: body_bounds.x,
                    y: body_bounds.y + row_offset - self.header_height,
                    width: body_bounds.width,
                    height: self.row_height,
                };

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

                    let mut cell_status = CellStatus::Default;
                    if cursor.is_over(row_bounds) {
                        cell_status = CellStatus::Hovered
                    }

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
                                cell_layout,
                                cursor,
                                viewport,
                            );
                        }
                    });
                }
            }
        });

        if self.has_header {
            let header_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: self.header_height,
            };

            // Clipping body cells to table header bounds
            renderer.with_layer(header_bounds, |renderer| {
                // render header background
                renderer.fill_quad(
                    Quad {
                        bounds: header_bounds,
                        border: Border {
                            color: Color::TRANSPARENT,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    table_style.header_background,
                );

                for (((cell, cell_layout), column_id), column_offset) in self
                    .header_cells
                    .iter()
                    .zip(header_cell_layouts)
                    .zip(self.columns.iter().map(|column| &column.id))
                    .zip(&self.column_offsets)
                {
                    // Cell bounds need to be intersected with table header bounds in case the current
                    // cell is the right most column header one.
                    let Some(cell_bounds) = cell_layout.bounds().intersection(&header_bounds)
                    else {
                        continue;
                    };

                    let mut cell_status = CellStatus::Default;
                    if cursor.is_over(cell_bounds) {
                        cell_status = CellStatus::Hovered;
                    }

                    let cell_style =
                        theme.cell_style(&self.cell_class, cell_status, CellType::Body);

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
                                cell_layout,
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
                                border: Border {
                                    color: Color::TRANSPARENT,
                                    ..Default::default()
                                },
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
                        border: Border {
                            color: Color::TRANSPARENT,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    table_style.header_body_separator,
                );
            });
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        _cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        match event {
            iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                let delta_y = match delta {
                    iced::mouse::ScrollDelta::Lines { x: _, y } => *y,
                    iced::mouse::ScrollDelta::Pixels { x: _, y } => *y,
                };

                state.offset_y += delta_y * self.row_height * -0.7;
                state.offset_y = state.offset_y.clamp(
                    0.0,
                    (self.row_height * self.records.len() as f32
                        - (layout.bounds().height - self.header_height))
                        .max(0.0),
                );

                shell.invalidate_layout();
                shell.request_redraw();
                shell.capture_event();
            }
            _ => {}
        }
    }
}
