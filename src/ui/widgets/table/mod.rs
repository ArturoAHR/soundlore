use std::ops::Range;

use iced::{
    Element, Length, Padding,
    advanced::renderer::{self},
    alignment,
    widget::Space,
};

use crate::ui::widgets::table::state::Identifiable;

pub mod state;
pub mod style;
pub mod widget;

pub use style::*;

pub struct Table<'a, T, Message, Theme, Renderer = iced::Renderer>
where
    T: Identifiable,
    Theme: Catalog,
{
    width: Length,
    height: Length,
    header_height: f32,
    row_height: f32,
    scroll_width: f32,

    has_header: bool,
    columns: Vec<Column<'a, T, Message, Theme, Renderer>>,
    records: &'a [T],

    visible_row_range: Range<usize>,
    header_cells: Vec<Element<'a, Message, Theme, Renderer>>,
    body_cells: Vec<Element<'a, Message, Theme, Renderer>>,

    column_offsets: Vec<f32>,
    row_offsets: Vec<f32>,

    cell_padding: Padding,
    header_cell_padding: Padding,

    table_class: Theme::TableClass<'a>,
    scroll_class: Theme::ScrollClass<'a>,
    body_row_class: Theme::BodyRowClass<'a>,
    cell_class: Theme::CellClass<'a>,
}

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn new(
        mut columns: Vec<Column<'a, T, Message, Theme, Renderer>>,
        records: &'a [T],
    ) -> Self {
        let has_header = columns.iter().any(|column| column.header.is_some());
        let header_height = if has_header { 35.0 } else { 0.0 };

        let mut header_cells = Vec::new();
        if has_header {
            header_cells = Vec::with_capacity(columns.len());

            for column in &mut columns {
                header_cells.push(column.header.take().unwrap_or(Space::new().into()))
            }
        }

        Self {
            header_height,
            has_header,

            width: Length::Fill,
            height: Length::Fill,
            row_height: 30.0,
            scroll_width: 12.0,

            columns,
            records,

            visible_row_range: 0..0,
            header_cells,
            body_cells: Vec::new(),

            column_offsets: Vec::new(),
            row_offsets: Vec::new(),

            header_cell_padding: [0.0, 8.0].into(),
            cell_padding: [0.0, 8.0].into(),

            table_class: Theme::default_table(),
            scroll_class: Theme::default_scroll(),
            body_row_class: Theme::default_body_row(),
            cell_class: Theme::default_cell(),
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

    pub fn scroll_width(mut self, scroll_width: impl Into<f32>) -> Self {
        self.scroll_width = scroll_width.into();

        self
    }

    pub fn cell_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.cell_padding = padding.into();

        self
    }

    pub fn header_cell_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.header_cell_padding = padding.into();

        self
    }

    pub fn style(mut self, function: impl Fn(&Theme) -> TableStyle + 'a) -> Self
    where
        Theme::TableClass<'a>: From<TableStyleFn<'a, Theme>>,
    {
        self.table_class = (Box::new(function) as TableStyleFn<'a, Theme>).into();

        self
    }

    pub fn scroll_style(
        mut self,
        function: impl Fn(&Theme, ScrollState) -> ScrollStyle + 'a,
    ) -> Self
    where
        Theme::ScrollClass<'a>: From<ScrollStyleFn<'a, Theme>>,
    {
        self.scroll_class = (Box::new(function) as ScrollStyleFn<'a, Theme>).into();

        self
    }

    pub fn body_row_style(
        mut self,
        function: impl Fn(&Theme, BodyRowStatus, usize) -> BodyRowStyle + 'a,
    ) -> Self
    where
        Theme::BodyRowClass<'a>: From<BodyRowStyleFn<'a, Theme>>,
    {
        self.body_row_class = (Box::new(function) as BodyRowStyleFn<'a, Theme>).into();

        self
    }

    pub fn cell_style(
        mut self,
        function: impl Fn(&Theme, CellStatus, CellType) -> CellStyle + 'a,
    ) -> Self
    where
        Theme::CellClass<'a>: From<CellStyleFn<'a, Theme>>,
    {
        self.cell_class = (Box::new(function) as CellStyleFn<'a, Theme>).into();

        self
    }
}

pub fn table<'a, T, Message, Theme, Renderer>(
    columns: Vec<Column<'a, T, Message, Theme, Renderer>>,
    records: &'a [T],
) -> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    Table::new(columns, records)
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
    header_padding: Option<Padding>,
    cell_padding: Option<Padding>,
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
        cell_padding: None,
        header_padding: None,
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

    pub fn cell_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.cell_padding = Some(padding.into());

        self
    }

    pub fn header_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.header_padding = Some(padding.into());

        self
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
