use iced::Rectangle;

pub const MINIMUM_SCROLL_THUMB_LENGTH: f32 = 40.0;
pub const SCROLL_THUMB_PADDING_WIDTH_RATIO: f32 = 0.25; // 25% of the scroll width

/// Gets the scroll thumb bounds based on the provided sizes, returns `None` if the table
/// cannot scroll. It is assumed the scroll height matches the visible height
// TODO: Add horizontal scroll alternative version.
pub fn get_scroll_thumb_bounds(
    scroll_bounds: Rectangle,
    total_scrollable_content_length: f32,
    scroll_offset: f32,
) -> Option<Rectangle> {
    if total_scrollable_content_length <= scroll_bounds.height
        || total_scrollable_content_length <= 0.1
    {
        return None;
    }

    let scroll_thumb_height = (scroll_bounds.height
        * (scroll_bounds.height / total_scrollable_content_length))
        .max(MINIMUM_SCROLL_THUMB_LENGTH);

    let scroll_thumb_padding = scroll_bounds.width * SCROLL_THUMB_PADDING_WIDTH_RATIO;
    let scroll_thumb_width = scroll_bounds.width - scroll_thumb_padding * 2.0;

    let scroll_offset_ratio =
        (scroll_offset / (total_scrollable_content_length - scroll_bounds.height)).min(1.0);
    let scroll_thumb_offset = (scroll_bounds.height - scroll_thumb_height) * scroll_offset_ratio;

    Some(Rectangle {
        x: scroll_bounds.x + scroll_thumb_padding,
        y: scroll_bounds.y + scroll_thumb_offset,
        width: scroll_thumb_width,
        height: scroll_thumb_height,
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const TEST_SCROLL_BOUNDS: Rectangle = Rectangle {
        height: 1000.0,
        width: 100.0,
        x: 0.0,
        y: 0.0,
    };

    #[test]
    fn should_get_vertical_scroll_thumb_bounds() {
        let scroll_thumb_bounds =
            get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 10000.0, 0.0).unwrap();

        assert_eq!(
            Rectangle {
                width: 50.0,
                height: 100.0,
                x: 25.0,
                y: 0.0
            },
            scroll_thumb_bounds
        )
    }

    #[test]
    fn should_get_vertical_scroll_thumb_bounds_with_scroll_offset() {
        let scroll_thumb_bounds =
            get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 10000.0, 100.0).unwrap();

        assert_eq!(
            Rectangle {
                width: 50.0,
                height: 100.0,
                x: 25.0,
                y: 10.0 // Scrolled 1% of the available scroll height
            },
            scroll_thumb_bounds
        )
    }

    #[test]
    fn should_get_vertical_scroll_thumb_bounds_with_scroll_offset_at_the_end() {
        let scroll_thumb_bounds =
            get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 10000.0, 10000.0).unwrap();

        assert_eq!(
            Rectangle {
                width: 50.0,
                height: 100.0,
                x: 25.0,
                y: 900.0
            },
            scroll_thumb_bounds
        )
    }

    #[test]
    fn should_get_vertical_scroll_thumb_bounds_with_scroll_offset_and_clamp_excessive_offsets() {
        let scroll_thumb_bounds =
            get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 10000.0, 20000.0).unwrap();

        assert_eq!(
            Rectangle {
                width: 50.0,
                height: 100.0,
                x: 25.0,
                y: 900.0
            },
            scroll_thumb_bounds
        )
    }

    #[test]
    fn should_get_vertical_scroll_thumb_bounds_with_scroll_thumb_height_clamped_for_very_long_contents()
     {
        let scroll_thumb_bounds =
            get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 100000.0, 0.0).unwrap();

        assert_eq!(
            Rectangle {
                width: 50.0,
                height: MINIMUM_SCROLL_THUMB_LENGTH,
                x: 25.0,
                y: 0.0
            },
            scroll_thumb_bounds
        )
    }

    #[test]
    fn should_not_get_vertical_scroll_thumb_bounds_if_visible_content_and_total_content_length_are_equal()
     {
        let scroll_thumb_bounds = get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 1000.0, 100.0);

        assert!(scroll_thumb_bounds.is_none())
    }

    #[test]
    fn should_not_get_vertical_scroll_thumb_bounds_if_table_visible_content_length_is_longer_than_total_content_length()
     {
        let scroll_thumb_bounds = get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 900.0, 100.0);

        assert!(scroll_thumb_bounds.is_none())
    }

    #[test]
    fn should_not_get_vertical_scroll_thumb_bounds_if_total_content_length_is_too_small() {
        let scroll_thumb_bounds = get_scroll_thumb_bounds(TEST_SCROLL_BOUNDS, 0.1, 100.0);

        assert!(scroll_thumb_bounds.is_none())
    }
}
