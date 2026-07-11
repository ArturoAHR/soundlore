pub fn get_file_name(sample_rate: u32, channels: u16, format: &str) -> String {
    let channel_count_name = match channels {
        1 => "mono",
        2 => "stereo",
        _ => unreachable!(),
    };

    format!("{sample_rate}_{channel_count_name}.{format}")
}
