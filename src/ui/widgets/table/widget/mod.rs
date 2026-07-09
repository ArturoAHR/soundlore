use iced::{
    Event, Length, Rectangle, Size,
    advanced::{
        Clipboard, Layout, Shell,
        layout::{Limits, Node},
        mouse::Cursor,
        renderer::{self},
        widget::{Tree, Widget, tree},
    },
};

use crate::ui::widgets::table::{Catalog, Table, state::Identifiable};

use crate::ui::widgets::table::state::State;

pub mod draw;
pub mod layout;
pub mod update;

use draw::*;
use layout::*;
use update::*;

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

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        layout(self, tree, renderer, limits)
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
        draw(self, tree, renderer, theme, style, layout, cursor, viewport)
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
        update(
            self, tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }
}
