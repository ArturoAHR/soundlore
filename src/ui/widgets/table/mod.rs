use std::{collections::HashSet, ops::Range};

use iced::{
    Border, Color, Element, Length, Point, Rectangle, Size,
    advanced::{
        Layout,
        layout::{Limits, Node},
        mouse::Cursor,
        renderer::{self, Quad},
        widget::{Tree, Widget, tree},
    },
    alignment,
    border::Radius,
    widget::Space,
};

use crate::ui::{
    utils::table::{
        column::{ColumnWidth, get_column_widths},
        virtualization::get_visible_range,
    },
    widgets::table::state::{HEADERS_ROW_IDENTIFIER, Identifiable},
};

use crate::ui::widgets::table::state::State;

pub mod state;

pub struct Table<'a, T, Message, Theme, Renderer = iced::Renderer>
where
    T: Identifiable,
    Theme: Catalog,
{
    width: Length,
    height: Length,
    header_height: f32,
    row_height: f32,

    has_header: bool,
    columns: Vec<Column<'a, T, Message, Theme, Renderer>>,
    records: &'a [T],

    visible_row_range: Range<usize>,
    header_cells: Vec<Element<'a, Message, Theme, Renderer>>,
    body_cells: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Theme: Catalog,
{
    pub fn new(columns: Vec<Column<'a, T, Message, Theme, Renderer>>, records: &'a [T]) -> Self {
        let has_header = columns.iter().any(|column| column.header.is_some());
        let header_height = if has_header { 40.0 } else { 0.0 };

        Self {
            header_height,
            has_header,

            width: Length::Fill,
            height: Length::Fill,
            row_height: 40.0,

            columns,
            records,

            visible_row_range: 0..0,
            header_cells: Vec::new(),
            body_cells: Vec::new(),
        }
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();

        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();

        self
    }

    pub fn header_height(mut self, header_height: impl Into<f32>) -> Self {
        if self.has_header {
            self.header_height = header_height.into();
        }

        self
    }

    pub fn row_height(mut self, row_height: impl Into<f32>) -> Self {
        self.row_height = row_height.into();

        self
    }
}

pub fn table<'a, T, Message, Theme, Renderer>(
    columns: Vec<Column<'a, T, Message, Theme, Renderer>>,
    records: &'a [T],
) -> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Theme: Catalog,
{
    Table::new(columns, records)
}

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

        if self.has_header {
            self.header_cells = Vec::with_capacity(self.columns.len());

            for column in &mut self.columns {
                self.header_cells
                    .push(column.header.take().unwrap_or(Space::new().into()))
            }
        }

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

        // Child Node Generation

        let mut nodes = Vec::new();

        let mut cell_offsets_x = Vec::new();
        let mut column_width_offset_sum = 0.0;
        for column in self.columns.iter() {
            cell_offsets_x.push(column_width_offset_sum);
            column_width_offset_sum += column.width;
        }

        let mut cell_offsets_y = Vec::new();
        let mut row_height_offset_sum =
            self.header_height - self.row_height * (state.offset_y / self.row_height).fract();
        for _ in self.visible_row_range.clone() {
            cell_offsets_y.push(row_height_offset_sum);
            row_height_offset_sum += self.row_height;
        }

        if self.has_header {
            for ((header_cell, column), offset_x) in self
                .header_cells
                .iter_mut()
                .zip(&self.columns)
                .zip(&cell_offsets_x)
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

                node = node.move_to(Point::new(*offset_x, 0.0)).align(
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

        for (visible_row_number, (record_id, offset_y)) in self.records
            [self.visible_row_range.clone()]
        .iter()
        .map(|record| record.id())
        .zip(cell_offsets_y.iter())
        .enumerate()
        {
            let row_body_cell_range = visible_row_number * self.columns.len()
                ..(visible_row_number + 1) * self.columns.len();

            row_ids.insert(record_id);

            for (body_cell, (column, offset_x)) in self.body_cells[row_body_cell_range]
                .iter_mut()
                .zip(self.columns.iter().zip(&cell_offsets_x))
            {
                let limits = Limits::new(Size::ZERO, Size::new(column.width, self.row_height));
                let mut tree = state
                    .cell_state
                    .get_mut_or_insert(record_id, &column.id, body_cell);

                let mut node = body_cell
                    .as_widget_mut()
                    .layout(&mut tree, renderer, &limits);

                node = node.move_to(Point::new(*offset_x, *offset_y)).align(
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                border: Border {
                    color: Color::WHITE,
                    radius: Radius::default(),
                    width: 1.0,
                },
                ..Default::default()
            },
            Color::BLACK,
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
            for (visible_row_number, row_id) in self.records[self.visible_row_range.clone()]
                .iter()
                .map(|record| record.id())
                .enumerate()
            {
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

                    // Clipping cell contents to cell bounds
                    renderer.with_layer(cell_bounds, |renderer| {
                        if let Some(cell_state) = state.cell_state.get(row_id, column_id) {
                            cell.as_widget().draw(
                                cell_state,
                                renderer,
                                theme,
                                style,
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
                for ((cell, cell_layout), column_id) in self
                    .header_cells
                    .iter()
                    .zip(header_cell_layouts)
                    .zip(self.columns.iter().map(|column| &column.id))
                {
                    // Cell bounds need to be intersected with table header bounds in case the current
                    // cell is the right most column header one.
                    let Some(cell_bounds) = cell_layout.bounds().intersection(&header_bounds)
                    else {
                        continue;
                    };

                    // Clipping cell contents to cell bounds
                    renderer.with_layer(cell_bounds, |renderer| {
                        if let Some(cell_state) =
                            state.cell_state.get(HEADERS_ROW_IDENTIFIER, column_id)
                        {
                            cell.as_widget().draw(
                                cell_state,
                                renderer,
                                theme,
                                style,
                                cell_layout,
                                cursor,
                                viewport,
                            );
                        }
                    });
                }
            });
        }
    }
}

pub struct Column<'a, T, Message, Theme, Renderer = iced::Renderer> {
    id: String,
    header: Option<Element<'a, Message, Theme, Renderer>>,
    view: Box<dyn Fn(&T) -> Element<'a, Message, Theme, Renderer> + 'a>,
    width: f32,
    min_width: f32,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    resizable: bool,
    sortable: bool,
}

pub fn column<'a, T, E, Message, Theme, Renderer>(
    id: String,
    header: Option<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(&T) -> E + 'a,
) -> Column<'a, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    Column {
        id,
        header,
        view: Box::new(move |data| view(data).into()),
        width: 100.0,
        min_width: 20.0,
        align_x: alignment::Horizontal::Left,
        align_y: alignment::Vertical::Center,
        resizable: false,
        sortable: false,
    }
}

impl<'a, T, Message, Theme, Renderer> Column<'a, T, Message, Theme, Renderer> {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();

        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = width.into();

        self
    }

    pub fn min_width(mut self, min_width: impl Into<f32>) -> Self {
        self.min_width = min_width.into();

        self
    }

    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = alignment.into();

        self
    }

    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.align_y = alignment.into();

        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;

        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;

        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {}

/// Theme catalog for a table
pub trait Catalog {
    /// Item class of the catalog
    type Class<'a>;

    /// The default class produced by the catalog
    fn default<'a>() -> Self::Class<'a>;

    /// The style of the class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// Styling function for a table widget.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl<Theme> From<Style> for StyleFn<'_, Theme> {
    fn from(style: Style) -> Self {
        Box::new(move |_theme| style)
    }
}

impl<'a, T, Message, Theme, Renderer> From<Table<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(table: Table<'a, T, Message, Theme, Renderer>) -> Self {
        Self::new(table)
    }
}
