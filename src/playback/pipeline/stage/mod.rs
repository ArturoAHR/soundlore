use crate::playback::pipeline::{
    AudioPipelineCommand, AudioPipelineCommandOutcome, AudioPipelineError,
};

pub mod channel_converter;
pub mod decoder;
pub mod resampler;

/// Represents one step in the pipeline process to transform from track file to ready to play samples.
pub enum AudioPipelineStage<Configuration> {
    Source(Box<dyn AudioPipelineSourceStage<Configuration>>),
    Join(Box<dyn AudioPipelineJoinStage<Configuration>>),
    Process(Box<dyn AudioPipelineProcessStage<Configuration>>),
    Output(Box<dyn AudioPipelineOutputStage<Configuration>>),
}

pub enum AudioTrackPipelineStage<Configuration> {
    Source(Box<dyn AudioPipelineSourceStage<Configuration>>),
    Process(Box<dyn AudioPipelineProcessStage<Configuration>>),
}

/// Wrapper for produced samples to allow next stages to know if they should perform end
/// of track procedures such as flushing.
pub enum AudioPipelineSamples {
    Chunk(Vec<f32>),
    End(Vec<f32>),
}

impl AudioPipelineSamples {
    /// Creates a new `AudioPipelineSamples` value with the same variant as the current instance.
    #[must_use]
    pub fn new_like(&self, samples: Vec<f32>) -> Self {
        match self {
            Self::Chunk(_) => Self::Chunk(samples),
            Self::End(_) => Self::End(samples),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Chunk(samples) | Self::End(samples) => samples.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AsRef<Vec<f32>> for AudioPipelineSamples {
    fn as_ref(&self) -> &Vec<f32> {
        match self {
            Self::Chunk(samples) | Self::End(samples) => samples,
        }
    }
}

impl AsRef<[f32]> for AudioPipelineSamples {
    fn as_ref(&self) -> &[f32] {
        match self {
            Self::Chunk(samples) | Self::End(samples) => samples,
        }
    }
}

/// Abstract audio pipeline base stage, do not use in `AudioPipelineStage` enum.
pub trait AudioPipelineBaseStage<Configuration> {
    /// Used to determined if the current stage should process or not.
    fn is_enabled(&self, configuration: &Configuration) -> bool;

    /// Handles incoming pipeline thread commands.
    fn handle_command(
        &mut self,
        _: &Configuration,
        _: &AudioPipelineCommand,
    ) -> Result<Option<AudioPipelineCommandOutcome>, AudioPipelineError> {
        Ok(None)
    }
}

/// Starting stages that produce samples.
pub trait AudioPipelineSourceStage<Configuration>: AudioPipelineBaseStage<Configuration> {
    fn process_stage(
        &mut self,
        configuration: &Configuration,
    ) -> Result<AudioPipelineSamples, AudioPipelineError>;
}

/// Normal transition stages that modify the input samples.
pub trait AudioPipelineProcessStage<Configuration>: AudioPipelineBaseStage<Configuration> {
    fn process_stage(
        &mut self,
        configuration: &Configuration,
        samples: AudioPipelineSamples,
    ) -> Result<AudioPipelineSamples, AudioPipelineError>;
}

/// Stages that join more than one samples vectors.
pub trait AudioPipelineJoinStage<Configuration>: AudioPipelineBaseStage<Configuration> {
    fn process_stage(
        &mut self,
        configuration: &Configuration,
        tracks_samples: Vec<AudioPipelineSamples>,
    ) -> Result<AudioPipelineSamples, AudioPipelineError>;
}

/// End stage that consumes the product of all the previous stages before it.
pub trait AudioPipelineOutputStage<Configuration>: AudioPipelineBaseStage<Configuration> {
    fn process_stage(
        &mut self,
        configuration: &Configuration,
        samples: AudioPipelineSamples,
    ) -> Result<(), AudioPipelineError>;
}
