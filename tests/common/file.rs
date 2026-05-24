use std::{
    env,
    fs::create_dir_all,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::common::generator::generate_audio_file_fixtures;

// TODO: Add recursive path where there are audio files in sub-folders when recursive scan is implemented.
pub struct AudioFileFixturesPath {
    pub all_formats: PathBuf,
    pub metadata_variants: PathBuf,
    pub corrupt: PathBuf,
    pub partially_corrupt: PathBuf,
}

pub static AUDIO_FILE_FIXTURES_PATH: LazyLock<AudioFileFixturesPath> = LazyLock::new(|| {
    let fixture_root = get_fixture_root();

    if !fixture_root.exists() {
        create_dir_all(&fixture_root).expect("Couldn't create audio files fixture directory");

        generate_audio_file_fixtures(&fixture_root);
    }

    AudioFileFixturesPath {
        all_formats: fixture_root.join("all_formats"),
        metadata_variants: fixture_root.join("metadata_variants"),
        corrupt: fixture_root.join("corrupt"),
        partially_corrupt: fixture_root.join("partially_corrupt"),
    }
});

/// Hash based off the generator file source code, to ensure that no stale fixtures are used
pub static FIXTURE_HASH: LazyLock<String> = LazyLock::new(|| {
    let generator_source_code = include_str!("./generator.rs");

    let mut hasher = DefaultHasher::new();
    generator_source_code.hash(&mut hasher);

    format!("{:016x}", hasher.finish())
});

fn get_fixture_root() -> PathBuf {
    let base = env!("CARGO_TARGET_TMPDIR");

    Path::new(&base).join("audio").join(&*FIXTURE_HASH)
}
