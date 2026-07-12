#![allow(clippy::float_cmp)]
// TODO: Implement a robust epsilon based assert comparison for tests with floats.

use iced::Rectangle;

pub fn get_table_grid_bounds(bounds: Rectangle, scroll_width: f32) -> Rectangle {
    Rectangle {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width - scroll_width,
        height: bounds.height,
    }
}

pub fn get_table_header_bounds(grid_bounds: Rectangle, header_height: f32) -> Rectangle {
    Rectangle {
        x: grid_bounds.x,
        y: grid_bounds.y,
        width: grid_bounds.width,
        height: header_height,
    }
}

pub fn get_table_body_bounds(grid_bounds: Rectangle, header_height: f32) -> Rectangle {
    Rectangle {
        x: grid_bounds.x,
        y: grid_bounds.y + header_height,
        width: grid_bounds.width,
        height: grid_bounds.height - header_height,
    }
}

pub fn get_table_body_row_bounds(
    body_bounds: Rectangle,
    header_height: f32,
    row_height: f32,
    row_offset: f32,
) -> Rectangle {
    Rectangle {
        x: body_bounds.x,
        y: body_bounds.y + row_offset - header_height,
        width: body_bounds.width,
        height: row_height,
    }
}

pub fn get_table_scroll_bounds(bounds: Rectangle, scroll_width: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - scroll_width,
        y: bounds.y,
        width: scroll_width,
        height: bounds.height,
    }
}

pub fn get_effective_scroll_area_bounds(scroll_bounds: Rectangle, header_height: f32) -> Rectangle {
    Rectangle {
        x: scroll_bounds.x,
        y: scroll_bounds.y + header_height,
        width: scroll_bounds.width,
        height: scroll_bounds.height - header_height,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn should_segment_table_bounds() {
        let header_height = 50.0;
        let row_height = 25.0;
        let row_offset = 50.0;
        let scroll_width = 50.0;
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 500.0,
        };

        let grid_bounds = get_table_grid_bounds(bounds, scroll_width);
        let header_bounds = get_table_header_bounds(grid_bounds, header_height);
        let body_bounds = get_table_body_bounds(grid_bounds, header_height);
        let body_row_bounds =
            get_table_body_row_bounds(body_bounds, header_height, row_height, row_offset);
        let scroll_bounds = get_table_scroll_bounds(bounds, scroll_width);
        let effective_scroll_area_bounds =
            get_effective_scroll_area_bounds(scroll_bounds, header_height);

        assert_eq!(
            grid_bounds,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 950.0,
                height: 500.0,
            }
        );

        assert_eq!(
            header_bounds,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 950.0,
                height: 50.0,
            }
        );

        assert_eq!(
            body_bounds,
            Rectangle {
                x: 0.0,
                y: 50.0,
                width: 950.0,
                height: 450.0
            }
        );

        assert_eq!(
            body_row_bounds,
            Rectangle {
                x: 0.0,
                y: 50.0,
                width: 950.0,
                height: 25.0
            }
        );

        assert_eq!(
            scroll_bounds,
            Rectangle {
                x: 950.0,
                y: 0.0,
                width: 50.0,
                height: 500.0
            }
        );

        assert_eq!(
            effective_scroll_area_bounds,
            Rectangle {
                x: 950.0,
                y: 50.0,
                width: 50.0,
                height: 450.0
            }
        );

        assert_eq!(
            header_bounds.height, header_height,
            "Header height is not being respected when calculating header bounds."
        );
        assert_eq!(
            scroll_bounds.width, scroll_width,
            "Scroll width is not being respected when calculating scroll bounds"
        );
        assert_eq!(
            effective_scroll_area_bounds.width, scroll_width,
            "Scroll width is not being respected when calculating effective scroll area bounds"
        );
        assert_eq!(
            bounds,
            grid_bounds.union(&scroll_bounds),
            "Union between grid bounds and scroll bounds does not yield the full bounds."
        );
        assert_eq!(
            grid_bounds,
            header_bounds.union(&body_bounds),
            "Union between header bounds and body bounds does not yield the grid bounds"
        );
        assert_eq!(
            bounds.width,
            body_bounds.union(&scroll_bounds).width,
            "Width of union of body bounds and scroll bounds does not equal full bounds width."
        );
        assert_eq!(
            body_bounds.height, effective_scroll_area_bounds.height,
            "Height of effective scroll area does not equal to body height."
        );
        assert_eq!(
            header_bounds.height,
            scroll_bounds.height - effective_scroll_area_bounds.height,
            "Difference between scroll bounds and effective scroll area isn't header height."
        );
    }
}
