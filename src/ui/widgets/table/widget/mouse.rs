use iced::{Point, Rectangle, advanced::mouse::Click, mouse};

use crate::ui::widgets::table::widget::{
    bounds::{
        get_effective_scroll_area_bounds, get_table_body_bounds, get_table_grid_bounds,
        get_table_header_bounds, get_table_scroll_bounds,
    },
    scroll::get_scroll_thumb_bounds,
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
    Scroll {
        scroll_area_offset: Option<f32>,
    },
    ScrollThumb {
        scroll_area_start_offset: f32,
        scroll_area_end_offset: f32,
    },
}

impl TableArea {
    pub fn get_position_table_area(
        position: Point,
        bounds: Rectangle,
        header_height: f32,
        scroll_width: f32,
        total_scrollable_content_height: f32,
        scroll_offset: f32,
    ) -> Option<Self> {
        let grid_bounds = get_table_grid_bounds(bounds, scroll_width);
        let header_bounds = get_table_header_bounds(grid_bounds, header_height);
        let body_bounds = get_table_body_bounds(grid_bounds, header_height);
        let scroll_bounds = get_table_scroll_bounds(bounds, scroll_width);
        let effective_scroll_area_bounds =
            get_effective_scroll_area_bounds(scroll_bounds, header_height);
        let scroll_thumb_bounds = get_scroll_thumb_bounds(
            effective_scroll_area_bounds,
            total_scrollable_content_height,
            scroll_offset,
        );

        if header_bounds.contains(position) {
            return Some(Self::Header);
        }

        if body_bounds.contains(position) {
            return Some(Self::Body);
        }

        if let Some(scroll_thumb_bounds) = scroll_thumb_bounds
            && scroll_thumb_bounds.contains(position)
        {
            let scroll_area_start_offset = position.y - scroll_thumb_bounds.y;
            let scroll_area_end_offset = scroll_thumb_bounds.height - scroll_area_start_offset;

            return Some(Self::ScrollThumb {
                scroll_area_start_offset,
                scroll_area_end_offset,
            });
        }

        if scroll_bounds.contains(position) {
            return Some(Self::Scroll {
                scroll_area_offset: scroll_thumb_bounds.map_or_else(
                    || None,
                    |scroll_thumb_bounds| Some(scroll_thumb_bounds.height / 2.0),
                ),
            });
        }

        None
    }
}
