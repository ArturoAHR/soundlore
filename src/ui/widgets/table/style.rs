use iced::{Background, Border, Color};

#[derive(Debug, Clone, Copy)]
pub struct TableStyle {
    /// Base table background color.
    pub background: Background,
    /// Border around the entire table.
    pub border: Border,
    /// Background color for table header.
    pub header_background: Background,
    /// Background color for the line dividing header and body.
    pub header_body_separator: Background,
    /// Background color for the line between each header.
    pub header_separator_x: Background,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollStyle {
    pub vertical_scroll: RailStyle,
    // TODO: Add horizontal rail when horizontal scroll is added.
}

#[derive(Debug, Clone, Copy)]
pub struct RailStyle {
    /// Background color of the scroll rail.
    pub background: Background,
    /// Background color of the thumb of the scroll.
    pub thumb_background: Background,
    /// Border around the scroll thumb.
    pub thumb_border: Border,
}

pub struct ScrollState {
    pub vertical_scroll_status: ScrollStatus,
    // TODO: Add horizontal scroll status when horizontal scroll is added.
}

/// Scrollbar Status, if there is not enough content to scroll the scrollbar will be disabled.
pub enum ScrollStatus {
    Disabled,
    Default,
    Hovered,
    Dragged,
}

#[derive(Debug, Clone, Copy)]
pub struct BodyRowStyle {
    /// Background color of the table body
    pub background: Background,
}

pub enum BodyRowStatus {
    Default,
    Hovered,
    Selected,
}

#[derive(Debug, Clone, Copy)]
pub struct CellStyle {
    /// Text color override for cell contents
    pub text_color: Color,
}

pub enum CellType {
    Header,
    Body,
}

pub enum CellStatus {
    Default,
    /// Status for hovering over a header cell or a body row.
    Hovered,
    /// Status for the header cell of a sorting column and selected rows in the body.
    Selected,
}

/// Theme catalog for a table
pub trait Catalog {
    /// Item class of the catalog
    type TableClass<'a>;
    type ScrollClass<'a>;
    type BodyRowClass<'a>;
    type CellClass<'a>;

    /// The default class produced by the catalog
    fn default_table<'a>() -> Self::TableClass<'a>;
    fn default_scroll<'a>() -> Self::ScrollClass<'a>;
    fn default_body_row<'a>() -> Self::BodyRowClass<'a>;
    fn default_cell<'a>() -> Self::CellClass<'a>;

    /// The style of the class with the given status.
    fn table_style(&self, class: &Self::TableClass<'_>) -> TableStyle;
    fn scroll_style(&self, class: &Self::ScrollClass<'_>, state: ScrollState) -> ScrollStyle;
    fn body_row_style(
        &self,
        class: &Self::BodyRowClass<'_>,
        status: BodyRowStatus,
        row_number: usize,
    ) -> BodyRowStyle;
    fn cell_style(
        &self,
        class: &Self::CellClass<'_>,
        status: CellStatus,
        cell_type: CellType,
    ) -> CellStyle;
}

/// Styling function for a table widget.
pub type TableStyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> TableStyle + 'a>;
pub type ScrollStyleFn<'a, Theme> = Box<dyn Fn(&Theme, ScrollState) -> ScrollStyle + 'a>;
pub type BodyRowStyleFn<'a, Theme> = Box<dyn Fn(&Theme, BodyRowStatus, usize) -> BodyRowStyle + 'a>;
pub type CellStyleFn<'a, Theme> = Box<dyn Fn(&Theme, CellStatus, CellType) -> CellStyle + 'a>;

impl<Theme> From<TableStyle> for TableStyleFn<'_, Theme> {
    fn from(style: TableStyle) -> Self {
        Box::new(move |_theme| style)
    }
}

impl<Theme> From<ScrollStyle> for ScrollStyleFn<'_, Theme> {
    fn from(style: ScrollStyle) -> Self {
        Box::new(move |_theme, _state| style)
    }
}

impl<Theme> From<BodyRowStyle> for BodyRowStyleFn<'_, Theme> {
    fn from(style: BodyRowStyle) -> Self {
        Box::new(move |_theme, _status, _row_number| style)
    }
}

impl<Theme> From<CellStyle> for CellStyleFn<'_, Theme> {
    fn from(style: CellStyle) -> Self {
        Box::new(move |_theme, _status, _type| style)
    }
}
