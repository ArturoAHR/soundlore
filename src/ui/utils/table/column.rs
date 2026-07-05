use tracing::{instrument, warn};

#[cfg(test)]
#[path = "column_test.rs"]
mod column_test;

pub const IMPRECISION_TOLERANCE: f64 = 0.2;

#[derive(Debug, Clone)]
pub struct ColumnWidthSpecification {
    width: f64,
    /// Minimum width of a resizable column, ignored if the column is not resizable.
    min_width: f64,
    resizable: bool,
}

#[instrument(level = "debug")]
pub fn get_column_widths(
    container_width: f64,
    mut columns: Vec<ColumnWidthSpecification>,
) -> Vec<f64> {
    if columns.is_empty() {
        return vec![];
    };

    // If for some reason resizable column widths fall below minimum width, correct it for calculation.
    columns.iter_mut().for_each(|column| {
        if column.width < column.min_width && column.resizable {
            column.width = column.min_width
        }
    });

    let column_width_sum = columns.iter().map(|column| column.width).sum::<f64>();

    // Column widths are within acceptable tolerance of container width.
    if column_width_sum < container_width + IMPRECISION_TOLERANCE
        && container_width - IMPRECISION_TOLERANCE < column_width_sum
    {
        return columns.iter().map(|column| column.width).collect();
    }

    // Columns widths are less than available container width.
    if column_width_sum < container_width {
        let fixed_column_widths = columns
            .iter()
            .filter_map(|column| {
                if !column.resizable {
                    Some(column.width)
                } else {
                    None
                }
            })
            .sum::<f64>();

        let resizable_column_widths = column_width_sum - fixed_column_widths;

        if resizable_column_widths <= 0.1 {
            warn!("Not enough resizable column width, returning widths as is.");

            return Vec::from_iter(columns.iter().map(|column| column.width));
        }

        let growth_ratio = (container_width - fixed_column_widths) / resizable_column_widths;

        return columns
            .iter()
            .map(|column| {
                if column.resizable {
                    column.width * growth_ratio
                } else {
                    column.width
                }
            })
            .collect();
    }

    // Columns overflow the current container width, we reduce width proportional to the shrink capacity of the column
    // if the capacity for shrinking is not enough we simply return the set of minimum widths and fixed widths.

    let column_min_width_sum = columns
        .iter()
        .map(|column| {
            if column.resizable {
                column.min_width
            } else {
                column.width
            }
        })
        .sum::<f64>();

    if column_min_width_sum >= container_width {
        warn!("Minimum widths of each column are larger than container width, overflowing...");

        return columns
            .iter()
            .map(|column| {
                if column.resizable {
                    column.min_width
                } else {
                    column.width
                }
            })
            .collect();
    }

    let column_shrink_needed = column_width_sum - container_width;
    let columns_shrink_capacity = columns.iter().map(|column| {
        if column.resizable {
            column.width - column.min_width
        } else {
            0.0
        }
    });
    let column_shrink_capacity_sum = columns_shrink_capacity.clone().sum::<f64>();

    return columns
        .iter()
        .zip(columns_shrink_capacity)
        .map(|(column, column_shrink_capacity)| {
            if column.resizable {
                column.width
                    - (column_shrink_capacity / column_shrink_capacity_sum) * column_shrink_needed
            } else {
                column.width
            }
        })
        .collect();
}
