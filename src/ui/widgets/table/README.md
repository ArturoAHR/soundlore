# Table

The table custom widget implements the iced `Widget` trait to be able to work seamlessly in iced rendering system, it implements the necessary features to support tens of thousands of rows in a performant manner.

These are the features that are supported:

- Table Body Virtualization
- Row Selection:
  - Normal click selects a row.
  - Shift click selects all the rows between an anchor and the clicked row.
  - Ctrl click toggles selection.
  - Ctrl Shift click includes all the rows between the anchor and clicked row in the selection.
  - Ctrl + A while the table and the window are focused to select all rows.
- Scrollbar Interaction:
  - Scroll wheel and track pad scrolling
  - Click and dragging the scroll thumb and the scroll bar track
- Column Header Selection

These interactions are tested with emulator and snapshot tests, when running the tests locally these will create untracked images to help diagnose snapshot mismatch issues with the hash in the `/snapshots/images` directory.

## Usage

The widget receives a set of references to records which are used to render the contents of the table cells from its corresponding row in the table, for a value to be representable in the table it needs to implement the `Identifiable` trait, the id obtained through the `id` trait method should be unique as row selection depends on that to properly work, suffice to say these borrowed records are never cloned but it requires that the records live as long as the widget does for that frame.

```rs
    table(
        columns,
        tracks
    )
    .selected_rows(&self.selected_track_ids)
    .on_row_select(Message::TrackRowSelected)
    .on_row_double_click(Message::TrackRowDoubleClicked)
    .on_header_cell_click(Message::ColumnHeaderCellClicked),
```

The table helper function receives two things, a list of references to the records and a vector of columns, each containing a closure that represents how the cells of that column are rendered based on the record of the row.

```rs
let columns = vec![
    table::column(
        TrackTableColumn::Artist,
        Some(text("Artist").into()),
        |track: &Track| {
            ellipsized_text(track.artist.as_deref().unwrap_or("Unknown"))
                .wrapping(text::Wrapping::None)
        },
    )
    //...
]
```

## Virtualization & State Management

This table is able to hold lists of records that go beyond the thousands and still remain performant because we are not rendering all the rows except the ones that are visible, this technique is called virtualization, we simply take into account the current offset of the table and based on that and the table body height we determine the amount of rows that are visible in the body, this approach is possible thanks to all rows being the same height, which makes it almost trivial to calculate the range of records whose rows we need to render.

Since virtualization constantly shifts the order of the table body cells, we cannot use the default state management system iced uses for widget children, since these states depend on the order the children are and the children themselves are created in an upstream step in the rendering process. So instead we use a custom state management system that holds the state of each of the cells keyed by `RowId` and `ColumnId` and we perform the diff ourselves when calculating the layout.

Besides diffing the cell state we prune the state of the cells that are outside of view, meaning that when something is scrolled out of view, we will lose its state. Right now this is a measure to keep memory consumption of large lists of records manageable.

## Column Width Resolution

When column widths are not sufficient to cover the table or exceed the table's width, we increase them maintaining the proportions of the widths that were passed in initially and reduce them proportional to how far they are from the minimum width configured, as long as the columns are marked resizable. If we reach a critical point when reducing the width of the table in which we cannot reduce the width of any of the columns we overflow the columns horizontally, although there is no horizontal scroll as of now (and the strategy to deal with this overflow might change later so it is not required).

## Requesting Redraws and Invalidating Layout

We gate requesting redraws and invalidating the layout to specific events that are not tied to shell publishing in the `update` method of the widget, we invalidate the layout when the vertical offset in the widget state changes meaningfully (value  delta of more than 0.1px), and we request a redraw each time the cursor hovers over a different area than the current one highlighted.
