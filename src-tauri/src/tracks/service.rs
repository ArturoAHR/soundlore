use std::path::Path;

use std::fs::File;

use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AppError;

pub fn upsert_track(path: &Path) -> Result<(), AppError> {
    let track_file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(track_file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    if let Some(track) = format.tracks().first() {
        let params = &track.codec_params;
        println!("Sample rate: {:?}", params.sample_rate);
        println!("Channels:    {:?}", params.channels);
        println!("Duration:    {:?}", params.n_frames);
    }

    // Metadata tags
    if let Some(metadata) = format.metadata().current() {
        for tag in metadata.tags() {
            println!("{}: {}", tag.key, tag.value);
        }
    }

    Ok(())
}
