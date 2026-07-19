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
    pub click: Option<Click>,
}

impl MouseInteraction {
    pub fn press_mouse_button(
        &mut self,
        mouse_button: mouse::Button,
        previous_click: Option<Click>,
    ) -> Click {
        let click = Click::new(self.position, mouse_button, previous_click);

        self.click = Some(click);

        click
    }

    pub fn release_mouse_button(&mut self) -> Option<Click> {
        self.click.take()
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
