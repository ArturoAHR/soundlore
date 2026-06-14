use std::sync::LazyLock;

pub static SUPPORTED_FILE_FORMATS: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| vec!["wav", "mp3", "ogg", "aac", "flac", "m4a", "aiff"]);

pub static SUPPORTED_SAMPLE_RATES: LazyLock<Vec<u32>> = LazyLock::new(|| vec![48000, 44100]);
pub static SUPPORTED_CHANNEL_COUNTS: LazyLock<Vec<u16>> = LazyLock::new(|| vec![1, 2]);
