use std::sync::mpsc::Sender;

use tracing::error;

pub struct AudioPipelineEventEmitter {
    event_sender: Sender<AudioPipelineEvent>,
}

pub enum AudioPipelineEvent {
    Exited,
    DecodingEnded,
    EndOfTrack,
}

impl AudioPipelineEventEmitter {
    pub fn new(event_sender: Sender<AudioPipelineEvent>) -> Self {
        Self { event_sender }
    }

    pub fn emit(&self, event: AudioPipelineEvent) {
        match self.event_sender.send(event) {
            Ok(_) => {}
            Err(error) => {
                error!("Failed to send audio pipeline event: {error}");
            }
        }
    }
}
