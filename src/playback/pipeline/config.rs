use crate::{
    playback::pipeline::{event::AudioPipelineEventEmitter, AudioFormat},
    track::models::Track,
};

#[derive(Clone)]
pub struct AudioPipelineConfiguration {
    pub event_emitter: AudioPipelineEventEmitter,
    pub volume_percentage: u8,
    pub output: AudioFormat,
}

pub struct AudioTrackPipelineConfiguration {
    pub event_emitter: AudioPipelineEventEmitter,
    pub volume_percentage: u8,
    pub output: AudioFormat,
    pub track: Track,
}

impl AudioTrackPipelineConfiguration {
    pub fn new(pipeline_configuration: AudioPipelineConfiguration, track: Track) -> Self {
        Self {
            track,
            event_emitter: pipeline_configuration.event_emitter,
            volume_percentage: pipeline_configuration.volume_percentage,
            output: pipeline_configuration.output,
        }
    }
}
