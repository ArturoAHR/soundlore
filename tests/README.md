# Integration Testing

This is the integration testing suite, here we have tests related to app message handling and playback, the integration tests are all part of a single binary crate in `app/main.rs` to prevent dead code warnings.

## Testing utilities

These are the tools that we have for testing the app and the playback system:

- In memory SQLite database setup helpers
- Audio file fixture generation of several groups using `ffmpeg` (all formats, metadata variants, sample rate and channel variants, corrupted files)
- Test Playback Engine for headless deterministic playback.
- Manual `iced::Task` executor
- Test App Harnesses
