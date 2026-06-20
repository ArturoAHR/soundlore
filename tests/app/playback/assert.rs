use pretty_assertions::assert_eq;

pub fn assert_sample_count(
    sample_count: usize,
    expected_sample_count: usize,
    file_name: &str,
    tolerance_percentage: Option<usize>,
) {
    let format = file_name.rsplit_once('.').unwrap().1;

    let tolerance_percentage = tolerance_percentage.unwrap_or(0);

    if tolerance_percentage == 0 {
        match format {
            "ogg" | "mp3" | "wav" | "aiff" | "flac" => assert_eq!(
                sample_count, expected_sample_count,
                "Incorrect sample count for file: {file_name}."
            ),
            "m4a" | "aac" => assert!(
                sample_count >= expected_sample_count,
                "Insufficient sample count for file: {file_name}, expected at least {expected_sample_count} but got {sample_count}",
            ),
            _ => unreachable!(),
        }

        return;
    }
    let expected_sample_count_high_end =
        expected_sample_count + (expected_sample_count * tolerance_percentage) / 100;
    let expected_sample_count_lower_end =
        expected_sample_count - (expected_sample_count * tolerance_percentage) / 100;

    match format {
        "ogg" | "mp3" | "wav" | "aiff" | "flac" => assert!(
            expected_sample_count_high_end >= sample_count
                && sample_count >= expected_sample_count_lower_end,
            "Sample count is not within tolerance for file: {file_name} with sample count {sample_count} and range [{expected_sample_count_lower_end},{expected_sample_count_high_end}]."
        ),
        "m4a" | "aac" => assert!(
            sample_count >= expected_sample_count_lower_end,
            "Insufficient sample count for file: {file_name}, expected at least {expected_sample_count_lower_end} but got {sample_count}",
        ),
        _ => unreachable!(),
    }
}
