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
