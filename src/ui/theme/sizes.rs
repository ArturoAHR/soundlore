#[derive(Debug, Clone, Copy)]
pub struct Sizes {
    pub space: Spacing,
    pub font: Typography,
    pub border: Borders,
    pub component: ComponentSizes,
}

impl Sizes {
    pub const DEFAULT: Self = Self {
        space: Spacing {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            xxl: 24.0,
        },
        font: Typography {
            caption: 11.0,
            small: 12.0,
            body: 14.0,
            h3: 16.0,
            h2: 20.0,
            h1: 28.0,
        },
        border: Borders {
            width: 1.0,
            radius_xs: 2.0,
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_pill: 9999.0,
        },
        component: ComponentSizes {
            app_header_height: 32.0,
            nav_bar_height: 36.0,
            status_bar_height: 22.0,
            playback_bar_height: 72.0,

            table_row_height: 28.0,
            table_header_height: 32.0,
            table_column_min_width: 60.0,

            button_height: 28.0,
            input_height: 28.0,

            scrollbar_width: 8.0,
            pane_min_width: 160.0,
            pane_min_height: 80.0,

            progress_thickness: 4.0,
            progress_handle_diameter: 12.0,
            volume_bar_thickness: 80.0,

            tag_pill_height: 20.0,
            tagging_modal_pill_height: 32.0,

            icon_sm: 14.0,
            icon_md: 18.0,
            icon_lg: 32.0,
        },
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Typography {
    pub caption: f32,
    pub small: f32,
    pub body: f32,
    pub h3: f32,
    pub h2: f32,
    pub h1: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Borders {
    pub width: f32,
    pub radius_xs: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_pill: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentSizes {
    pub app_header_height: f32,
    pub nav_bar_height: f32,
    pub status_bar_height: f32,
    pub playback_bar_height: f32,

    pub table_row_height: f32,
    pub table_header_height: f32,
    pub table_column_min_width: f32,

    pub button_height: f32,
    pub input_height: f32,

    pub scrollbar_width: f32,
    pub pane_min_width: f32,
    pub pane_min_height: f32,

    pub progress_thickness: f32,
    pub progress_handle_diameter: f32,
    pub volume_bar_thickness: f32,

    pub tag_pill_height: f32,
    pub tagging_modal_pill_height: f32,

    pub icon_sm: f32,
    pub icon_md: f32,
    pub icon_lg: f32,
}
