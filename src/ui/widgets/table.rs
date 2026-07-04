use iced::{
    Border, Color, Element, Length, Rectangle, Size,
    advanced::{
        Layout,
        layout::{Limits, Node},
        mouse::Cursor,
        renderer::{self, Quad},
        widget::{Tree, Widget},
    },
    border::Radius,
};

pub struct Table {}

impl Table {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn table() -> Table {
    Table::new()
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Table
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            height: Length::Fill,
            width: Length::Fill,
        }
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
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

impl<Message, Theme, Renderer> From<Table> for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(table: Table) -> Self {
        Self::new(table)
    }
}
