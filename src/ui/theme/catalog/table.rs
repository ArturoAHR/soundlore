use iced::{Border, Color, border::Radius};

use crate::ui::{
    theme::{
        Theme,
        color::{darken, lighten},
    },
    widgets::table::{
        BodyRowStatus, BodyRowStyle, Catalog, CellStatus, CellStyle, CellType, RailStyle,
        ScrollState, ScrollStatus, ScrollStyle, TableStyle,
    },
};

pub type TableStyleFn<'a> = Box<dyn Fn(&Theme) -> TableStyle + 'a>;
pub type ScrollStyleFn<'a> = Box<dyn Fn(&Theme, ScrollState) -> ScrollStyle + 'a>;
pub type BodyRowStyleFn<'a> = Box<dyn Fn(&Theme, BodyRowStatus, usize) -> BodyRowStyle + 'a>;
pub type CellStyleFn<'a> = Box<dyn Fn(&Theme, CellStatus, CellType) -> CellStyle + 'a>;

impl Catalog for Theme {
    type TableClass<'a> = TableStyleFn<'a>;
    type ScrollClass<'a> = ScrollStyleFn<'a>;
    type BodyRowClass<'a> = BodyRowStyleFn<'a>;
    type CellClass<'a> = CellStyleFn<'a>;

    fn default_table<'a>() -> Self::TableClass<'a> {
        Box::new(|theme| TableStyle {
            background: theme.palette.surface.into(),
            border: Border {
                color: Color::TRANSPARENT,
                ..Default::default()
            },
            header_background: darken(theme.palette.surface, 0.3).into(),
            header_body_separator: theme.palette.border.into(),
            header_separator_x: theme.palette.border.into(),
        })
    }

    fn default_scroll<'a>() -> Self::ScrollClass<'a> {
        Box::new(|theme, state| {
            let default_scroller_color = theme.palette.surface;

            let scroller_color = match state.vertical_scroll_status {
                ScrollStatus::Disabled => Color::TRANSPARENT,
                ScrollStatus::Default => default_scroller_color,
                ScrollStatus::Hovered => lighten(default_scroller_color, 0.1),
                ScrollStatus::Dragged => lighten(default_scroller_color, 0.2),
            };

            ScrollStyle {
                vertical_scroll: RailStyle {
                    background: darken(theme.palette.surface, 0.3).into(),
                    thumb_background: scroller_color.into(),
                    thumb_border: Border {
                        radius: Radius::from(10.0),
                        ..Default::default()
                    },
                },
            }
        })
    }

    fn default_body_row<'a>() -> Self::BodyRowClass<'a> {
        Box::new(|theme, status, row_number| {
            let body_row_color = match status {
                BodyRowStatus::Default => {
                    if row_number % 2 == 0 {
                        theme.palette.surface
                    } else {
                        darken(theme.palette.surface, 0.15)
                    }
                }
                BodyRowStatus::Hovered => lighten(theme.palette.hover, 0.1),
                BodyRowStatus::Selected => theme.palette.selected,
            };

            BodyRowStyle {
                background: body_row_color.into(),
            }
        })
    }

    fn default_cell<'a>() -> Self::CellClass<'a> {
        Box::new(|theme, status, cell_type| {
            let text_color = match status {
                CellStatus::Default => match cell_type {
                    CellType::Header => darken(theme.palette.text, 0.5),
                    CellType::Body => theme.palette.text,
                },
                CellStatus::Hovered => match cell_type {
                    CellType::Header => darken(theme.palette.text_selected, 0.2),
                    CellType::Body => theme.palette.text_selected,
                },
                CellStatus::Selected => theme.palette.text_selected,
            };

            CellStyle {
                text_color: text_color.into(),
            }
        })
    }

    fn table_style(&self, class: &Self::TableClass<'_>) -> TableStyle {
        class(self)
    }

    fn scroll_style(&self, class: &Self::ScrollClass<'_>, state: ScrollState) -> ScrollStyle {
        class(self, state)
    }

    fn body_row_style(
        &self,
        class: &Self::BodyRowClass<'_>,
        status: BodyRowStatus,
        row_number: usize,
    ) -> BodyRowStyle {
        class(self, status, row_number)
    }

    fn cell_style(
        &self,
        class: &Self::CellClass<'_>,
        status: CellStatus,
        cell_type: CellType,
    ) -> CellStyle {
        class(self, status, cell_type)
    }
}
