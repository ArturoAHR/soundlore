use iced::{Point, Rectangle, advanced::mouse::Click, mouse};

use crate::ui::widgets::table::widget::bounds::{
    get_table_body_bounds, get_table_grid_bounds, get_table_header_bounds, get_table_scroll_bounds,
};

#[derive(Debug, Clone, Copy)]
pub struct TableClick {
    pub table_area: Option<TableArea>,
    pub current_position: Point,
    pub click: Click,
}

impl TableClick {
    pub fn new(
        position: Point,
        button: mouse::Button,
        table_area: Option<TableArea>,
        previous_table_click: Option<Self>,
    ) -> Self {
        let click = Click::new(
            position,
            button,
            previous_table_click.map(|table_click| table_click.click),
        );

        Self {
            click,
            current_position: position,
            table_area,
        }
    }

    pub fn get_current_position_delta(&self, position: Point) -> Point {
        Point {
            x: self.current_position.x - position.x,
            y: self.current_position.y - position.y,
        }
    }

    pub fn get_initial_position_delta(&self, position: Point) -> Point {
        let initial_position = self.click.position();

        Point {
            x: initial_position.x - position.x,
            y: initial_position.y - position.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TableArea {
    Header,
    Body,
    Scroll,
}

impl TableArea {
    pub fn get_position_table_area(
        position: Point,
        bounds: Rectangle,
        header_height: f32,
        scroll_width: f32,
    ) -> Option<Self> {
        let grid_bounds = get_table_grid_bounds(bounds, scroll_width);
        let header_bounds = get_table_header_bounds(grid_bounds, header_height);
        let body_bounds = get_table_body_bounds(grid_bounds, header_height);
        let scroll_bounds = get_table_scroll_bounds(bounds, scroll_width);

        if header_bounds.contains(position) {
            return Some(Self::Header);
        }

        if body_bounds.contains(position) {
            return Some(Self::Body);
        }

        if scroll_bounds.contains(position) {
            return Some(Self::Scroll);
        }

        None
    }
}
