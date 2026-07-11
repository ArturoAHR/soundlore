use crate::playback::pipeline::AudioPipelineError;

/// Gets a user friendly error based of the specific error received
//TODO: Add message controller error messages and remove this allow directive
#[allow(clippy::match_single_binding)]
pub fn get_audio_controller_error_message(error: AudioPipelineError) -> String {
    match error {
        error => format!("{error}"),
    }
}
