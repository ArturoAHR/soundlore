use tracing::{error, instrument};

use crate::playback::pipeline::thread::AudioPipelineThreadEvent;

pub type EventSender = std::sync::mpsc::Sender<AudioPipelineThreadEvent>;

#[derive(Clone)]
pub struct AudioPipelineEventEmitter {
    event_sender: EventSender,
}

impl AudioPipelineEventEmitter {
    pub fn new(event_sender: EventSender) -> Self {
        Self { event_sender }
    }

    #[instrument(skip(self), ret, level = "debug")]
    pub fn emit(&self, event: AudioPipelineThreadEvent) {
        match self.event_sender.send(event) {
            Ok(()) => {}
            Err(error) => {
                error!("Failed to send audio pipeline event: {error}");
            }
        }
    }
}
