use crate::constants::{MIN_HORIZONTAL_SPLIT_PANE_HEIGHT, MIN_VERTICAL_SPLIT_PANE_WIDTH};

pub fn seconds_to_timestamp(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds / 60) % 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

pub fn is_vertical_pane_split_valid(
    explorer_main_ratio: f64,
    main_queue_track_information_ratio: f64,
    window_width: f64,
) -> bool {
    let explorer_pane_ratio = explorer_main_ratio;
    let main_pane_ratio = (1.0 - explorer_main_ratio) * main_queue_track_information_ratio;
    let queue_track_information_pane_ratio =
        (1.0 - explorer_main_ratio) * (1.0 - main_queue_track_information_ratio);

    let explorer_pane_width = explorer_pane_ratio * window_width as f64;
    let main_pane_width = main_pane_ratio * window_width as f64;
    let queue_track_information_pane_width =
        queue_track_information_pane_ratio * window_width as f64;

    explorer_pane_width >= MIN_VERTICAL_SPLIT_PANE_WIDTH
        && main_pane_width >= MIN_VERTICAL_SPLIT_PANE_WIDTH
        && queue_track_information_pane_width >= MIN_VERTICAL_SPLIT_PANE_WIDTH
}

pub fn is_horizontal_pane_split_valid(
    queue_track_information_ratio: f64,
    window_height: f64,
) -> bool {
    let queue_pane_ratio = queue_track_information_ratio;
    let track_information_ratio = 1.0 - queue_track_information_ratio;

    let queue_pane_width = queue_pane_ratio * window_height;
    let track_information_width = track_information_ratio * window_height;

    queue_pane_width >= MIN_HORIZONTAL_SPLIT_PANE_HEIGHT
        && track_information_width >= MIN_HORIZONTAL_SPLIT_PANE_HEIGHT
}
