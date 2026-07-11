use iced::Rectangle;

/// Gets the scroll thumb bounds based on the provided sizes.
/// TODO: Add horizontal scroll alternative version.
pub fn get_scroll_thumb_bounds(
    scroll_bounds: Rectangle,
    total_scrollable_content_length: f32,
    visible_content_length: f32,
    scroll_offset: f32,
) -> Rectangle {
    let scroll_thumb_height = (scroll_bounds.height
        * (visible_content_length / total_scrollable_content_length))
        .max(40.0);

    let scroll_thumb_padding = scroll_bounds.width * 0.25;
    let scroll_thumb_width = scroll_bounds.width - scroll_thumb_padding * 2.0;

    let scroll_thumb_offset = (scroll_bounds.height - scroll_thumb_height)
        * (scroll_offset / (total_scrollable_content_length - visible_content_length));

    Rectangle {
        x: scroll_bounds.x + scroll_thumb_padding,
        y: scroll_bounds.y + scroll_thumb_offset,
        width: scroll_thumb_width,
        height: scroll_thumb_height,
    }
}
