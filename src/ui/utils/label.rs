/// Converts the provided seconds amount into a human readable string
/// ```
/// # use soundlore_lib::ui::utils::label::format_duration;
/// # use pretty_assertions::assert_eq;
/// assert_eq!(&format_duration(605), "10:05");
/// assert_eq!(&format_duration(3670), "1:01:10");
/// ```
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds / 60) % 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}
