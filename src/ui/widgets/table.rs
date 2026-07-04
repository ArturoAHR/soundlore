use iced::{
    Border, Color, Element, Length, Rectangle, Size,
    advanced::{
        Layout,
        layout::{Limits, Node},
        mouse::Cursor,
        renderer::{self, Quad},
        widget::{Tree, Widget, tree},
    },
    alignment,
    border::Radius,
};

pub struct Table<'a, T, Message, Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
{
    width: Length,
    height: Length,
    header_height: f32,
    row_height: f32,

    header_cells: Vec<Element<'a, Message, Theme, Renderer>>,

    columns: Vec<Column<'a, T, Message, Theme, Renderer>>,
    records: &'a [T],
}

// pub struct ColumnConfiguration {
//     width: Length,
//     align_x: alignment::Horizontal,
//     align_y: alignment::Vertical,
//     resizable: bool,
//     sortable: bool,
// }

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    pub fn new(columns: Vec<Column<'a, T, Message, Theme, Renderer>>, records: &'a [T]) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            header_height: 40.0,
            row_height: 40.0,

            header_cells: Vec::new(),

            columns,
            records,
        }
    }
}

struct State {
    offset_y: f32,
}

pub fn table<'a, T, Message, Theme, Renderer>(
    columns: Vec<Column<'a, T, Message, Theme, Renderer>>,
    records: &'a [T],
) -> Table<'a, T, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    Table::new(columns, records)
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, T, Message, Theme, Renderer>
where
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
        tree::State::new(State { offset_y: 0.0 })
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let offset_y = tree.state.downcast_mut::<State>();

        Node::new(limits.max())
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
        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                border: Border {
                    color: Color::BLACK,
                    radius: Radius::default(),
                    width: 1.0,
                },
                ..Default::default()
            },
            Color::WHITE,
        );
    }
}

pub struct Column<'a, T, Message, Theme, Renderer = iced::Renderer> {
    header: Option<Element<'a, Message, Theme, Renderer>>,
    view: Box<dyn Fn(&T) -> Element<'a, Message, Theme, Renderer> + 'a>,
    width: Length,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    resizable: bool,
    sortable: bool,
}

pub fn column<'a, T, E, Message, Theme, Renderer>(
    header: Option<Element<'a, Message, Theme, Renderer>>,
    view: impl Fn(&T) -> E + 'a,
) -> Column<'a, T, Message, Theme, Renderer>
where
    T: 'a,
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    Column {
        header,
        view: Box::new(move |data| view(data).into()),
        width: Length::Fill,
        align_x: alignment::Horizontal::Left,
        align_y: alignment::Vertical::Center,
        resizable: false,
        sortable: false,
    }
}

impl<'a, T, Message, Theme, Renderer> Column<'a, T, Message, Theme, Renderer> {
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();

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
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(table: Table<'a, T, Message, Theme, Renderer>) -> Self {
        Self::new(table)
    }
}
