use tracing::{instrument, warn};

#[cfg(test)]
#[path = "column_test.rs"]
mod column_test;

pub const IMPRECISION_TOLERANCE: f64 = 0.2;

#[derive(Debug, Clone)]
pub enum ColumnWidth {
    Resizable { width: f64, min_width: f64 },
    Fixed { width: f64 },
}

impl ColumnWidth {
    pub fn get_width(&self) -> f64 {
        match self {
            Self::Fixed { width }
            | Self::Resizable {
                width,
                min_width: _,
            } => *width,
        }
    }

    pub fn get_min_width(&self) -> f64 {
        match self {
            Self::Fixed { width } => *width,
            Self::Resizable {
                width: _,
                min_width,
            } => *min_width,
        }
    }

    pub fn get_shrink_capacity(&self) -> f64 {
        match self {
            Self::Fixed { width: _ } => 0.0,
            Self::Resizable { width, min_width } => width - min_width,
        }
    }
}

#[instrument(level = "debug")]
pub fn get_column_widths(container_width: f64, mut column_widths: Vec<ColumnWidth>) -> Vec<f64> {
    if column_widths.is_empty() {
        return vec![];
    }

    // If for some reason resizable column widths fall below minimum width, correct it for calculation.
    for column in &mut column_widths {
        if let ColumnWidth::Resizable { width, min_width } = column
            && width < min_width
        {
            *width = *min_width;
        }
    }

    let column_width_sum = column_widths
        .iter()
        .map(ColumnWidth::get_width)
        .sum::<f64>();

    // Column widths are within acceptable tolerance of container width.
    if column_width_sum < container_width + IMPRECISION_TOLERANCE
        && container_width - IMPRECISION_TOLERANCE < column_width_sum
    {
        return column_widths.iter().map(ColumnWidth::get_width).collect();
    }

    // Columns widths are less than available container width.
    if column_width_sum < container_width {
        let fixed_column_widths = column_widths
            .iter()
            .filter_map(|column_width| match column_width {
                ColumnWidth::Fixed { width } => Some(width),
                ColumnWidth::Resizable {
                    width: _,
                    min_width: _,
                } => None,
            })
            .sum::<f64>();

        let resizable_column_widths = column_width_sum - fixed_column_widths;

        if resizable_column_widths <= 0.1 {
            warn!("Not enough resizable column width, returning widths as is.");

            return column_widths.iter().map(ColumnWidth::get_width).collect();
        }

        let growth_ratio = (container_width - fixed_column_widths) / resizable_column_widths;

        return column_widths
            .iter()
            .map(|column| match column {
                ColumnWidth::Fixed { width } => *width,
                ColumnWidth::Resizable {
                    width,
                    min_width: _,
                } => *width * growth_ratio,
            })
            .collect();
    }

    // Columns overflow the current container width, we reduce width proportional to the shrink capacity of the column
    // if the capacity for shrinking is not enough we simply return the set of minimum widths and fixed widths.

    let column_min_width_sum = column_widths
        .iter()
        .map(ColumnWidth::get_min_width)
        .sum::<f64>();

    if column_min_width_sum >= container_width {
        warn!("Minimum widths of each column are larger than container width, overflowing...");

        return column_widths
            .iter()
            .map(ColumnWidth::get_min_width)
            .collect();
    }

    let column_shrink_needed = column_width_sum - container_width;
    let columns_shrink_capacity = column_widths.iter().map(ColumnWidth::get_shrink_capacity);
    let column_shrink_capacity_sum = columns_shrink_capacity.clone().sum::<f64>();

    column_widths
        .iter()
        .zip(columns_shrink_capacity)
        .map(|(column, column_shrink_capacity)| match column {
            ColumnWidth::Resizable {
                width,
                min_width: _,
            } => {
                width - (column_shrink_capacity / column_shrink_capacity_sum) * column_shrink_needed
            }
            ColumnWidth::Fixed { width } => *width,
        })
        .collect()
}
