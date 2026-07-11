use super::*;

fn generate_column_width_specifications(widths: &[f64], min_width: f64) -> Vec<ColumnWidth> {
    widths
        .iter()
        .map(|width| ColumnWidth::Resizable {
            width: *width,
            min_width,
        })
        .collect()
}

fn assert_column_widths(expected_column_widths: &[f64], actual_column_widths: &[f64]) {
    let verification_closure = |expected_column_widths: &[f64], actual_column_widths: &[f64]| {
        if expected_column_widths.len() != actual_column_widths.len() {
            return false;
        }

        for (expected_column_width, actual_column_width) in expected_column_widths
            .iter()
            .zip(actual_column_widths.iter())
        {
            if expected_column_width - IMPRECISION_TOLERANCE <= *actual_column_width
                && *actual_column_width <= expected_column_width + IMPRECISION_TOLERANCE
            {
                continue;
            }

            return false;
        }

        true
    };

    assert!(
        verification_closure(expected_column_widths, actual_column_widths),
        "Column widths do not match within {IMPRECISION_TOLERANCE:.2?} pixel tolerance:

        Expected: {expected_column_widths:.2?}
        Actual: {actual_column_widths:.2?}
        ",
    );
}

fn assert_container_width_fit(container_width: f64, column_widths: &[f64]) {
    let column_width_sum = column_widths.iter().sum::<f64>();

    assert!(
        container_width - IMPRECISION_TOLERANCE <= column_width_sum
            && column_width_sum <= container_width + IMPRECISION_TOLERANCE,
        "Column widths do not fit in the container width within {IMPRECISION_TOLERANCE:.2?} pixel tolerance:

        Expected: {container_width:.2?}
        Actual: {column_width_sum:.2?}
        ",
    );
}

#[test]
fn should_return_unchanged_widths_if_passed_in_columns_widths_fit_the_container() {
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 1000.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns);

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(&column_widths, &resulting_column_widths);
}

#[test]
fn should_return_unchanged_widths_if_passed_in_non_resizable_columns_widths_fit_the_container() {
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 1000.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);
    let fixed_columns = columns
        .iter()
        .map(|_| ColumnWidth::Fixed { width: 200.0 })
        .collect();

    let resulting_column_widths = get_column_widths(container_width, fixed_columns);

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(&column_widths, &resulting_column_widths);
}

#[test]
fn should_return_unchanged_widths_if_all_columns_are_not_resizable_and_the_sum_is_less_than_container_width()
 {
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 2000.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);
    let fixed_columns = columns
        .iter()
        .map(|_| ColumnWidth::Fixed { width: 200.0 })
        .collect();

    let resulting_column_widths = get_column_widths(container_width, fixed_columns);

    assert_column_widths(&column_widths, &resulting_column_widths);
}

#[test]
fn should_return_proportionally_increased_widths_if_column_width_sum_is_less_than_container_width()
{
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 2000.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns);

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &column_widths.map(|column_width| column_width * 2.0),
        &resulting_column_widths,
    );
}

#[test]
fn should_return_proportionally_increased_widths_if_column_width_sum_is_less_than_container_width_with_fixed_columns()
 {
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 2200.0;

    let mut columns = generate_column_width_specifications(&column_widths, 100.0);

    columns[0] = ColumnWidth::Fixed { width: 200.0 };

    let resulting_column_widths = get_column_widths(container_width, columns.clone());

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &[200.0, 400.0, 400.0, 400.0, 400.0, 400.0],
        &resulting_column_widths,
    );
}

#[test]
fn should_return_proportionally_increased_widths_if_column_width_sum_is_less_than_container_width_with_different_column_sizes()
 {
    let column_widths = [100.0, 200.0, 300.0, 400.0, 500.0];
    let container_width = 2000.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns);

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &[133.33, 266.67, 400.00, 533.33, 666.67],
        &resulting_column_widths,
    );
}

#[test]
fn should_return_proportionally_increased_widths_if_column_width_sum_is_less_than_container_width_with_different_column_sizes_and_fixed_columns()
 {
    let column_widths = [50.0, 100.0, 200.0, 300.0, 400.0, 500.0, 50.0];
    let container_width = 2100.0;

    let mut columns = generate_column_width_specifications(&column_widths, 100.0);

    columns[0] = ColumnWidth::Fixed { width: 50.0 };
    columns[6] = ColumnWidth::Fixed { width: 50.0 };

    let resulting_column_widths = get_column_widths(container_width, columns.clone());

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &[50.0, 133.33, 266.67, 400.00, 533.33, 666.67, 50.0],
        &resulting_column_widths,
    );
}

#[test]
fn should_return_proportionally_decreased_widths_if_column_width_sum_is_more_than_container_width()
{
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 750.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns);

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &column_widths.map(|column_width| column_width * 0.75),
        &resulting_column_widths,
    );
}

#[test]
fn should_return_proportionally_decreased_widths_if_column_width_sum_is_more_than_container_width_with_fixed_columns()
 {
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 950.0;

    let mut columns = generate_column_width_specifications(&column_widths, 100.0);

    columns[0] = ColumnWidth::Fixed { width: 200.0 };

    let resulting_column_widths = get_column_widths(container_width, columns.clone());

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &[200.0, 150.0, 150.0, 150.0, 150.0, 150.0],
        &resulting_column_widths,
    );
}

#[test]
fn should_return_proportionally_decreased_widths_if_column_width_sum_is_more_than_container_width_with_different_column_sizes()
 {
    let column_widths = [100.0, 200.0, 300.0, 400.0, 500.0];
    let container_width = 750.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns);

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &[100.00, 125.00, 150.00, 175.00, 200.00],
        &resulting_column_widths,
    );
}

#[test]
fn should_return_proportionally_decreased_widths_if_column_width_sum_is_more_than_container_width_with_different_column_sizes_and_fixed_columns()
 {
    let column_widths = [50.0, 100.0, 200.0, 300.0, 400.0, 500.0, 50.0];
    let container_width = 850.0;

    let mut columns = generate_column_width_specifications(&column_widths, 100.0);

    columns[0] = ColumnWidth::Fixed { width: 50.0 };
    columns[6] = ColumnWidth::Fixed { width: 50.0 };

    let resulting_column_widths = get_column_widths(container_width, columns.clone());

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &[50.0, 100.00, 125.00, 150.00, 175.00, 200.00, 50.0],
        &resulting_column_widths,
    );
}

#[test]
fn should_return_minimum_widths_if_all_columns_minimum_widths_sum_is_equal_to_container_width() {
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 500.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns.clone());

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &columns
            .iter()
            .map(ColumnWidth::get_min_width)
            .collect::<Vec<f64>>(),
        &resulting_column_widths,
    );
}

#[test]
fn should_return_minimum_widths_if_all_columns_minimum_widths_sum_is_larger_than_container_width() {
    let column_widths = [200.0, 200.0, 200.0, 200.0, 200.0];
    let container_width = 400.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns.clone());

    assert_column_widths(
        &columns
            .iter()
            .map(ColumnWidth::get_min_width)
            .collect::<Vec<f64>>(),
        &resulting_column_widths,
    );
}

#[test]
fn should_fix_resizable_widths_that_go_below_minimum_that_would_have_fit_the_container_container_otherwise()
 {
    let column_widths = [50.0, 50.0, 50.0, 50.0, 50.0];
    let container_width = 500.0;

    let columns = generate_column_width_specifications(&column_widths, 100.0);

    let resulting_column_widths = get_column_widths(container_width, columns);

    assert_container_width_fit(container_width, &resulting_column_widths);
    assert_column_widths(
        &[100.0, 100.0, 100.0, 100.0, 100.0],
        &resulting_column_widths,
    );
}
