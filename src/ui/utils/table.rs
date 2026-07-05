use std::ops::Range;

use tracing::error;

/// Gets the visible range of elements that we need to display in a table based on the vertical offset
/// and different heights
/// ```
/// # use soundlore_lib::ui::utils::table::get_visible_range;
/// // Need to show 5 rows for a container with size 120.0 and header with size 20.0.
/// assert_eq!(0..5, get_visible_range(120.0, 20.0, 20.0, 0.0));
/// ```
pub fn get_visible_range(
    container_height: f32,
    row_height: f32,
    header_height: f32,
    offset_y: f32,
) -> Range<usize> {
    if row_height <= 0.5 {
        error!(
            container_height = ?container_height,
            row_height = ?row_height,
            header_height = ?header_height,
            offset_y = ?offset_y,
            "Failed to determine visible range, row height is too small"
        );
        return 0..0;
    }

    let range_start = (offset_y / row_height).floor() as usize;

    let body_height = (container_height - header_height).max(0.0);
    let range_end = ((body_height + offset_y) / row_height).ceil() as usize;

    range_start..range_end
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_get_visible_range() {
        assert_eq!(0..5, get_visible_range(120.0, 20.0, 20.0, 0.0));
    }

    #[test]
    fn should_get_visible_range_with_exact_height_offset() {
        assert_eq!(1..6, get_visible_range(120.0, 20.0, 20.0, 20.0));
    }

    #[test]
    fn should_get_visible_range_with_partial_height_offset() {
        assert_eq!(0..6, get_visible_range(120.0, 20.0, 20.0, 10.0));
    }

    #[test]
    fn should_get_visible_range_without_header_height() {
        assert_eq!(0..6, get_visible_range(120.0, 20.0, 20.0, 10.0));
    }

    #[test]
    fn should_collapse_range_when_row_height_is_too_small() {
        assert_eq!(0..0, get_visible_range(120.0, 0.1, 20.0, 10.0));
    }

    #[test]
    fn should_handle_when_table_body_height_is_too_small() {
        assert_eq!(0..1, get_visible_range(10.0, 20.0, 20.0, 10.0));
    }
}
