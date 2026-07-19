use iced::{
    Point,
    advanced::mouse::Click,
    mouse::{self},
};

use crate::ui::widgets::table::state::TableIdentifier;

#[derive(Debug, Clone, Default)]
pub struct MouseInteraction {
    pub area: Option<TableArea>,
    pub position: Point,
    pub click: Option<TableClick>,
}

impl MouseInteraction {
    pub fn press_mouse_button(
        &mut self,
        mouse_button: mouse::Button,
        previous_click: Option<TableClick>,
    ) -> Click {
        let click = Click::new(
            self.position,
            mouse_button,
            previous_click.map(|table_click| table_click.click),
        );

        self.click = Some(TableClick {
            area: self.area.clone(),
            click,
        });

        click
    }

    pub fn release_mouse_button(&mut self) -> Option<TableClick> {
        self.click.take()
    }
}

#[derive(Debug, Clone)]
pub struct TableClick {
    pub area: Option<TableArea>,
    pub click: Click,
}

impl TableClick {
    pub fn new(area: Option<TableArea>, click: Click) -> Self {
        Self { area, click }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableArea {
    Header {
        column_id: Option<TableIdentifier>,
    },
    Body {
        row_id: Option<TableIdentifier>,
    },
    Scroll {
        scroll_area_offset: Option<f32>,
    },
    ScrollThumb {
        scroll_area_start_offset: f32,
        scroll_area_end_offset: f32,
    },
}
