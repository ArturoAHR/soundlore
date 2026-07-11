/// Checks that the pane widths of the main are above the minimum for the given ratios.
/// ```
/// # use soundlore_lib::ui::utils::pane::are_pane_widths_valid;
/// // Valid split widths (200, 400, 400).
/// assert!(are_pane_widths_valid(0.2, 0.5, 1000.0, 200.0));
/// // Invalid split widths (150, 700, 150).
/// assert!(!are_pane_widths_valid(0.15, 0.82, 1000.0, 200.0));
/// ```
pub fn are_pane_widths_valid(
    explorer_main_ratio: f64,
    main_queue_track_information_ratio: f64,
    window_width: f64,
    minimum_width: f64,
) -> bool {
    let explorer_pane_ratio = explorer_main_ratio;
    let main_pane_ratio = (1.0 - explorer_main_ratio) * main_queue_track_information_ratio;
    let queue_track_information_pane_ratio =
        (1.0 - explorer_main_ratio) * (1.0 - main_queue_track_information_ratio);

    let explorer_pane_width = explorer_pane_ratio * window_width;
    let main_pane_width = main_pane_ratio * window_width;
    let queue_track_information_pane_width = queue_track_information_pane_ratio * window_width;

    explorer_pane_width >= minimum_width
        && main_pane_width >= minimum_width
        && queue_track_information_pane_width >= minimum_width
}

/// Checks that the pane heights of the queue track information split are above the minimum for the given ratio.
/// ```
/// # use soundlore_lib::ui::utils::pane::are_pane_heights_valid;
/// // Valid split heights (500, 500).
/// assert!(are_pane_heights_valid(0.5, 1000.0, 200.0));
/// // Invalid split heights (30, 970).
/// assert!(!are_pane_heights_valid(0.1, 1000.0, 200.0));
/// ```
pub fn are_pane_heights_valid(
    queue_track_information_ratio: f64,
    window_height: f64,
    minimum_height: f64,
) -> bool {
    let queue_pane_ratio = queue_track_information_ratio;
    let track_information_ratio = 1.0 - queue_track_information_ratio;

    let queue_pane_height = queue_pane_ratio * window_height;
    let track_information_pane_height = track_information_ratio * window_height;

    queue_pane_height >= minimum_height && track_information_pane_height >= minimum_height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_pass_on_valid_pane_widths() {
        assert!(are_pane_widths_valid(0.2, 0.5, 1000.0, 200.0));
    }

    #[test]
    fn should_fail_on_invalid_explorer_pane_width() {
        assert!(!are_pane_widths_valid(0.19, 0.5, 1000.0, 200.0));
    }

    #[test]
    fn should_fail_on_invalid_main_pane_width() {
        assert!(!are_pane_widths_valid(0.9, 0.5, 1000.0, 200.0));
    }

    #[test]
    fn should_fail_on_invalid_queue_pane_width() {
        assert!(!are_pane_widths_valid(0.5, 0.8, 1000.0, 200.0));
    }

    #[test]
    fn should_pass_on_valid_pane_heights() {
        assert!(are_pane_heights_valid(0.5, 1000.0, 200.0));
    }

    #[test]
    fn should_fail_on_invalid_queue_pane_height() {
        assert!(!are_pane_heights_valid(0.10, 1000.0, 200.0));
    }

    #[test]
    fn should_fail_on_invalid_track_information_pane_height() {
        assert!(!are_pane_heights_valid(0.90, 1000.0, 200.0));
    }
}
