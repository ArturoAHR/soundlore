use std::{hash::Hash, ops::Range};

use iced::{
    Element, Event, Length, Padding, Rectangle, Size,
    advanced::renderer::{self},
    advanced::{
        Clipboard, Layout, Shell,
        layout::{Limits, Node},
        mouse::Cursor,
        widget::{Tree, Widget, tree},
    },
    alignment,
    widget::Space,
};
use rustc_hash::FxHashSet;

use crate::{
    traits::Identifiable,
    ui::{utils::table::column::ColumnWidth, widgets::table::state::State},
};

mod bounds;
mod draw;
mod event;
mod layout;
mod scroll;
pub mod state;
pub mod style;
mod update;

#[cfg(test)]
pub mod tests;

pub use style::*;

pub type OnRowSelectHandler<'a, RowId, Message> = Box<dyn Fn(FxHashSet<RowId>) -> Message + 'a>;

pub struct Table<'a, 'b, T, ColumnId, Message, Theme, Renderer = iced::Renderer>
where
    T: Identifiable + TableRow,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
    Theme: Catalog,
{
    width: Length,
    height: Length,
    header_height: f32,
    row_height: f32,
    scroll_width: f32,

    selected_rows: Option<&'a FxHashSet<T::Identifier>>,
    /// Returns the set of table body row identifiers that are currently selected every time the set changes.
    on_row_select: Option<OnRowSelectHandler<'a, T::Identifier, Message>>,
    on_header_cell_click: Option<Box<dyn Fn(ColumnId) -> Message + 'a>>,
    on_row_double_click: Option<Box<dyn Fn(T::Identifier) -> Message + 'a>>,

    has_header: bool,
    columns: Vec<Column<'a, T, ColumnId, Message, Theme, Renderer>>,
    records: Vec<&'b T>,

    visible_row_range: Range<usize>,
    header_cells: Vec<Element<'a, Message, Theme, Renderer>>,
    body_cells: Vec<Element<'a, Message, Theme, Renderer>>,

    column_offsets: Vec<f32>,
    row_offsets: Vec<f32>,

    cell_padding: Padding,
    header_cell_padding: Padding,

    class: Theme::TableClass<'a>,
    scroll_class: Theme::ScrollClass<'a>,
    body_row_class: Theme::BodyRowClass<'a>,
    cell_class: Theme::CellClass<'a>,
}

pub trait TableRow: Identifiable {
    fn header_row_id() -> Self::Identifier;
}

impl<'a, 'b, T, ColumnId, Message, Theme, Renderer>
    Table<'a, 'b, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
    'b: 'a,
{
    pub fn new(
        mut columns: Vec<Column<'a, T, ColumnId, Message, Theme, Renderer>>,
        records: Vec<&'b T>,
    ) -> Self {
        let has_header = columns.iter().any(|column| column.header.is_some());
        let header_height = if has_header { 35.0 } else { 0.0 };

        let mut header_cells = Vec::new();
        if has_header {
            header_cells = Vec::with_capacity(columns.len());

            for column in &mut columns {
                header_cells.push(column.header.take().unwrap_or_else(|| Space::new().into()));
            }
        }

        Self {
            header_height,
            has_header,

            width: Length::Fill,
            height: Length::Fill,
            row_height: 30.0,
            scroll_width: 12.0,

            selected_rows: None,
            on_row_select: None,
            on_header_cell_click: None,
            on_row_double_click: None,

            columns,
            records,

            visible_row_range: 0..0,
            header_cells,
            body_cells: Vec::new(),

            column_offsets: Vec::new(),
            row_offsets: Vec::new(),

            header_cell_padding: [0.0, 8.0].into(),
            cell_padding: [0.0, 8.0].into(),

            class: Theme::default_table(),
            scroll_class: Theme::default_scroll(),
            body_row_class: Theme::default_body_row(),
            cell_class: Theme::default_cell(),
        }
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();

        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();

        self
    }

    #[must_use]
    pub fn header_height(mut self, header_height: impl Into<f32>) -> Self {
        if self.has_header {
            self.header_height = header_height.into();
        }

        self
    }

    #[must_use]
    pub fn row_height(mut self, row_height: impl Into<f32>) -> Self {
        self.row_height = row_height.into();

        self
    }

    #[must_use]
    pub fn scroll_width(mut self, scroll_width: impl Into<f32>) -> Self {
        self.scroll_width = scroll_width.into();

        self
    }

    #[must_use]
    pub fn cell_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.cell_padding = padding.into();

        self
    }

    #[must_use]
    pub fn header_cell_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.header_cell_padding = padding.into();

        self
    }

    #[must_use]
    pub fn selected_rows(mut self, selected_rows: &'a FxHashSet<T::Identifier>) -> Self {
        self.selected_rows = Some(selected_rows);

        self
    }

    #[must_use]
    pub fn on_row_select(
        mut self,
        on_row_select: impl Fn(FxHashSet<T::Identifier>) -> Message + 'a,
    ) -> Self {
        self.on_row_select = Some(Box::new(on_row_select));

        self
    }

    #[must_use]
    pub fn on_row_double_click(
        mut self,
        on_row_double_click: impl Fn(T::Identifier) -> Message + 'a,
    ) -> Self {
        self.on_row_double_click = Some(Box::new(on_row_double_click));

        self
    }

    #[must_use]
    pub fn on_header_cell_click(
        mut self,
        on_header_cell_click: impl Fn(ColumnId) -> Message + 'a,
    ) -> Self {
        self.on_header_cell_click = Some(Box::new(on_header_cell_click));

        self
    }

    #[must_use]
    pub fn style(mut self, function: impl Fn(&Theme) -> TableStyle + 'a) -> Self
    where
        Theme::TableClass<'a>: From<TableStyleFn<'a, Theme>>,
    {
        self.class = (Box::new(function) as TableStyleFn<'a, Theme>).into();

        self
    }

    #[must_use]
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

    #[must_use]
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

    #[must_use]
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

impl<'a, 'b, T, ColumnId, Message, Theme, Renderer>
    From<Table<'a, 'b, T, ColumnId, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Identifiable + TableRow + 'static,
    T::Identifier: Hash + Eq + Clone + 'static,
    ColumnId: Hash + Eq + Clone + 'static,
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + 'a,
    'b: 'a,
{
    fn from(table: Table<'a, 'b, T, ColumnId, Message, Theme, Renderer>) -> Self {
        Self::new(table)
    }
}

pub type ColumnCellView<'a, T, Message, Theme, Renderer> =
    dyn Fn(&'a T) -> Element<'a, Message, Theme, Renderer> + 'a;

pub struct Column<'a, T, ColumnId, Message, Theme, Renderer = iced::Renderer> {
    id: ColumnId,
    header: Option<Element<'a, Message, Theme, Renderer>>,
    view: Box<ColumnCellView<'a, T, Message, Theme, Renderer>>,
    width: f32,
    min_width: f32,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    resizable: bool,
    sortable: bool,
    header_padding: Option<Padding>,
    cell_padding: Option<Padding>,
}

impl<T, ColumnId, Message, Theme, Renderer> Column<'_, T, ColumnId, Message, Theme, Renderer> {
    pub fn get_column_width(&self) -> ColumnWidth {
        if self.resizable {
            ColumnWidth::Resizable {
                width: From::<f32>::from(self.width),
                min_width: From::<f32>::from(self.min_width),
            }
        } else {
            ColumnWidth::Fixed {
                width: From::<f32>::from(self.width),
            }
        }
    }

    #[must_use]
    pub fn id(mut self, id: impl Into<ColumnId>) -> Self {
        self.id = id.into();

        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = width.into();

        self
    }

    #[must_use]
    pub fn min_width(mut self, min_width: impl Into<f32>) -> Self {
        self.min_width = min_width.into();

        self
    }

    #[must_use]
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.align_x = alignment.into();

        self
    }

    #[must_use]
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.align_y = alignment.into();

        self
    }

    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;

        self
    }

    #[must_use]
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;

        self
    }

    #[must_use]
    pub fn cell_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.cell_padding = Some(padding.into());

        self
    }

    #[must_use]
    pub fn header_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.header_padding = Some(padding.into());

        self
    }
}

impl<'a, 'b, T, ColumnId, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, 'b, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow + 'static,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone + 'static,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
    'b: 'a,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<T::Identifier, ColumnId>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<T::Identifier, ColumnId>::default())
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.layout_impl(tree, renderer, limits)
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
        self.draw_impl(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }
}

/// Creates an new Table with the given columns and one row for each record.
pub fn table<'a, 'b, T, ColumnId, Message, Theme, Renderer>(
    columns: Vec<Column<'a, T, ColumnId, Message, Theme, Renderer>>,
    records: Vec<&'b T>,
) -> Table<'a, 'b, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
    'b: 'a,
{
    Table::new(columns, records)
}

/// Creates a column with the given parameters, the view closure is used to determined how
/// we represent the record in that particular column cell in its row.
pub fn column<'a, T, E, ColumnId, Message, Theme, Renderer>(
    id: ColumnId,
    header: Option<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(&'a T) -> E + 'a,
) -> Column<'a, T, ColumnId, Message, Theme, Renderer>
where
    T: 'a,
    ColumnId: Hash + Eq,
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
