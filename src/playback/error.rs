use crate::playback::pipeline::AudioPipelineError;

/// Gets a user friendly error based of the specific error received
pub fn get_audio_controller_error_message(error: AudioPipelineError) -> String {
    let message = match error {
        error => format!("{error}"),
    };

    message.to_owned()
}
