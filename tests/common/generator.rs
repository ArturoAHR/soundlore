/* WARNING: Changes to this file will result in the regeneration of the fixtures on the next test run */

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use ffmpeg_sidecar::{command::FfmpegCommand, download::auto_download};
use std::fs::create_dir_all;

pub fn generate_audio_file_fixtures(path: &PathBuf) {
    auto_download().expect("Could not download ffmpeg to generate audio file fixtures.");

    let all_formats_files_path = path.join("all_formats");
    let metadata_variants_files_path = path.join("metadata_variants");
    let corrupt_files_path = path.join("corrupt");
    let partially_corrupt_files_path = path.join("partially_corrupt");
    let sample_rate_and_channels_variants_file_path = path.join("all_sample_rates_and_channels");

    generate_all_formats_files(&all_formats_files_path);
    generate_metadata_variants_files(&metadata_variants_files_path);
    generate_corrupt_files(&corrupt_files_path);
    generate_partially_corrupt_files(&partially_corrupt_files_path);
    generate_sample_rate_and_channels_variants_files(&sample_rate_and_channels_variants_file_path)
}

fn generate_all_formats_files(output_path: &PathBuf) {
    create_dir_all(&output_path)
        .expect("Could not create all formats audio file fixtures directory.");

    // WAV audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "pcm_s16le"]);
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.wav"));

    // MP3 audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(
        &mut arguments,
        vec!["-c:a", "libmp3lame", "-b:a", "128k", "-id3v2_version", "3"],
    );
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.mp3"));

    // OGG Vorbis audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.ogg"));

    // FLAC audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "flac"]);
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.flac"));

    // M4A audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "aac", "-b:a", "128k"]);
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.m4a"));

    // AAC audio file (Should drop all tags)
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "aac", "-b:a", "128k"]);
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.aac"));

    // AIFF
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(
        &mut arguments,
        vec!["-c:a", "pcm_s16be", "-write_id3v2", "1"],
    );
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.aiff"));
}

fn generate_metadata_variants_files(output_path: &PathBuf) {
    create_dir_all(&output_path)
        .expect("Could not create metadata variants audio file fixtures directory.");

    // No tags
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    run_ffmpeg(arguments, &output_path.join("no_tags.ogg"));

    // Only title tag
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    extend_arguments(&mut arguments, vec!["-metadata", "title=Only Title"]);
    run_ffmpeg(arguments, &output_path.join("only_title.ogg"));

    // Artist and album tags
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    extend_arguments(
        &mut arguments,
        vec![
            "-metadata",
            "artist=Solo Artist",
            "-metadata",
            "album=Solo Album",
        ],
    );
    run_ffmpeg(arguments, &output_path.join("artist_album.ogg"));

    // Slash form tags
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "flac"]);
    extend_arguments(
        &mut arguments,
        vec![
            "-metadata",
            "title=Slash Numerics",
            "-metadata",
            "track=3/12",
            "-metadata",
            "disc=1/2",
            "-metadata",
            "date=2020-05-13",
        ],
    );
    run_ffmpeg(arguments, &output_path.join("slash_numerics.flac"));

    // Unicode title with long untrimmed artist
    let long_artist = format!("   {}", "x".repeat(2048));
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "flac"]);
    extend_arguments(
        &mut arguments,
        vec![
            "-metadata",
            "title=日本語タイトル ⟨long⟩",
            "-metadata",
            &format!("artist={long_artist}"),
        ],
    );
    run_ffmpeg(arguments, &output_path.join("unicode_title.flac"));

    // Unicode title MP3
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(
        &mut arguments,
        vec!["-c:a", "libmp3lame", "-b:a", "128k", "-id3v2_version", "3"],
    );
    extend_arguments(
        &mut arguments,
        vec!["-metadata", "title=日本語タイトル ⟨long⟩"],
    );
    run_ffmpeg(arguments, &output_path.join("unicode_mp3.mp3"));

    // Unicode title OGG
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    extend_arguments(
        &mut arguments,
        vec!["-metadata", "title=日本語タイトル ⟨long⟩"],
    );
    run_ffmpeg(arguments, &output_path.join("unicode_ogg.ogg"));
}

fn generate_corrupt_files(output_path: &PathBuf) {
    create_dir_all(&output_path).expect("Could not create corrupt audio file fixtures directory.");

    // Not an audio file
    fs::write(output_path.join("not_audio.mp3"), b"plain text")
        .expect("Could not create invalid audio file");

    // Empty audio file
    fs::write(output_path.join("empty.flac"), b"").expect("Could not create empty audio file");

    // Truncated audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    let full_file_path = output_path.join("_full.ogg");
    run_ffmpeg(arguments, &full_file_path);

    let bytes = fs::read(&full_file_path).expect("Could not read full audio file to truncate");
    fs::write(output_path.join("truncated.ogg"), &bytes[..bytes.len() / 4])
        .expect("Could not create truncated audio file");

    fs::remove_file(&full_file_path).expect("could not remove the original full audio file");

    // Mislabeled
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    let real_file_path = output_path.join("_read.ogg");
    run_ffmpeg(arguments, &real_file_path);
    fs::rename(real_file_path, output_path.join("mislabeled.mp3"))
        .expect("Could not rename audio file to mislabel it");

    // Unsupported formats
    fs::write(output_path.join("text.txt"), b"plain text").expect("Could not create text file");
    fs::write(output_path.join("cover.jpg"), &[0xff, 0xd8, 0xff, 0xe0])
        .expect("Could not create image");

    // Hidden metadata / garbage
    fs::write(output_path.join(".DS_Store"), b"\x00\x00")
        .expect("Could not create hidden metadata file");
    fs::write(output_path.join("._01.flac"), b"garbage file")
        .expect("Could not create hidden garbage file");
}

fn generate_partially_corrupt_files(output_path: &PathBuf) {
    create_dir_all(output_path)
        .expect("Could not create partially corrupt audio file fixtures directory.");

    // WAV audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "pcm_s16le"]);
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.wav"));

    // MP3 audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(
        &mut arguments,
        vec!["-c:a", "libmp3lame", "-b:a", "128k", "-id3v2_version", "3"],
    );
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.mp3"));

    // OGG Vorbis audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    extend_arguments(&mut arguments, generate_full_metadata_tags_arguments());
    run_ffmpeg(arguments, &output_path.join("track.ogg"));

    // Not an audio file
    fs::write(output_path.join("not_audio.mp3"), b"plain text")
        .expect("Could not create invalid audio file");

    // Empty audio file
    fs::write(output_path.join("empty.flac"), b"").expect("Could not create empty audio file");

    // Truncated audio file
    let mut arguments = generate_default_file_creation_arguments();
    extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
    let full_file_path = output_path.join("_full.ogg");
    run_ffmpeg(arguments, &full_file_path);

    let bytes = fs::read(&full_file_path).expect("Could not read full audio file to truncate");
    fs::write(output_path.join("truncated.ogg"), &bytes[..bytes.len() / 4])
        .expect("Could not create truncated audio file");

    fs::remove_file(&full_file_path).expect("could not remove the original full audio file");
}

fn generate_sample_rate_and_channels_variants_files(output_path: &PathBuf) {
    create_dir_all(&output_path)
        .expect("Could not create all formats audio file fixtures directory.");

    let sample_rates = vec!["48000", "44100"];
    let channel_counts = vec!["1", "2"];

    for sample_rate in sample_rates.iter() {
        for channels in channel_counts.iter() {
            let channel_count_name = match *channels {
                "1" => "mono",
                "2" => "stereo",
                _ => "unknown",
            };

            let file_name = format!("{}_{}", sample_rate, channel_count_name);

            // WAV audio file
            let mut arguments = generate_default_file_creation_arguments();
            extend_arguments(&mut arguments, vec!["-c:a", "pcm_s16le"]);
            extend_arguments(&mut arguments, vec!["-ar", sample_rate, "-ac", channels]);
            run_ffmpeg(arguments, &output_path.join(format!("{}.wav", file_name)));

            // MP3 audio file
            let mut arguments = generate_default_file_creation_arguments();
            extend_arguments(
                &mut arguments,
                vec!["-c:a", "libmp3lame", "-b:a", "128k", "-id3v2_version", "3"],
            );
            extend_arguments(&mut arguments, vec!["-ar", sample_rate, "-ac", channels]);
            run_ffmpeg(arguments, &output_path.join("track.mp3"));

            // OGG Vorbis audio file
            let mut arguments = generate_default_file_creation_arguments();
            extend_arguments(&mut arguments, vec!["-c:a", "libvorbis"]);
            extend_arguments(&mut arguments, vec!["-ar", sample_rate, "-ac", channels]);
            run_ffmpeg(arguments, &output_path.join(format!("{}.ogg", file_name)));

            // FLAC audio file
            let mut arguments = generate_default_file_creation_arguments();
            extend_arguments(&mut arguments, vec!["-c:a", "flac"]);
            extend_arguments(&mut arguments, vec!["-ar", sample_rate, "-ac", channels]);
            run_ffmpeg(arguments, &output_path.join(format!("{}.flac", file_name)));

            // M4A audio file
            let mut arguments = generate_default_file_creation_arguments();
            extend_arguments(&mut arguments, vec!["-c:a", "aac", "-b:a", "128k"]);
            extend_arguments(&mut arguments, vec!["-ar", sample_rate, "-ac", channels]);
            run_ffmpeg(arguments, &output_path.join(format!("{}.m4a", file_name)));

            // AAC audio file (Should drop all tags)
            let mut arguments = generate_default_file_creation_arguments();
            extend_arguments(&mut arguments, vec!["-c:a", "aac", "-b:a", "128k"]);
            extend_arguments(&mut arguments, vec!["-ar", sample_rate, "-ac", channels]);
            run_ffmpeg(arguments, &output_path.join(format!("{}.aac", file_name)));

            // AIFF
            let mut arguments = generate_default_file_creation_arguments();
            extend_arguments(
                &mut arguments,
                vec!["-c:a", "pcm_s16be", "-write_id3v2", "1"],
            );
            extend_arguments(&mut arguments, vec!["-ar", sample_rate, "-ac", channels]);
            run_ffmpeg(arguments, &output_path.join(format!("{}.aiff", file_name)));
        }
    }
}

fn extend_arguments(arguments: &mut Vec<String>, new_arguments: Vec<impl Into<String> + Clone>) {
    arguments.extend(
        new_arguments
            .iter()
            .cloned()
            .map(|argument| argument.into())
            .collect::<Vec<String>>(),
    );
}

fn generate_default_file_creation_arguments() -> Vec<String> {
    generate_file_creation_arguments(1, 440)
}

fn generate_file_creation_arguments(seconds: u32, frequency: u32) -> Vec<String> {
    vec![
        "-f".into(),
        "lavfi".into(),
        "-i".into(),
        format!("sine=frequency={frequency}:duration={seconds}"),
    ]
}

fn generate_full_metadata_tags_arguments() -> Vec<String> {
    generate_metadata_arguments(vec![
        ("title", "Test Track"),
        ("artist", "Test Artist"),
        ("album", "Test Album"),
        ("album_artist", "Test Album Artist"),
        ("date", "2024"),
        ("genre", "Test Genre"),
        // Vorbis comments need -total fields, will be ignored in other containers.
        ("track", "3/12"),
        ("tracktotal", "12"),
        ("disc", "1/2"),
        ("disctotal", "2"),
        // ReplayGain
        ("replaygain_track_gain", "-6.54 dB"),
        ("replaygain_track_peak", "0.987654"),
        ("replaygain_album_gain", "-7.20 dB"),
        ("replaygain_album_peak", "0.999000"),
    ])
}

fn generate_metadata_arguments(
    metadata_arguments: Vec<(impl Into<String>, impl Into<String>)>,
) -> Vec<String> {
    metadata_arguments
        .into_iter()
        .flat_map(|(tag, value)| {
            [
                "-metadata".into(),
                format!("{}={}", tag.into(), value.into()),
            ]
        })
        .collect()
}

fn run_ffmpeg<I, S>(arguments: I, output_file_path: &Path)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = FfmpegCommand::new();
    command.arg("-y");

    for argument in arguments {
        command.arg(argument);
    }

    command
        .output(output_file_path.to_string_lossy())
        .spawn()
        .expect("Could not spawn ffmpeg command child process")
        .wait()
        .expect("Command failed during execution");
}
