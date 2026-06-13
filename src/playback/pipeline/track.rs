use std::vec;

use tracing::{instrument, trace, warn};

use crate::{
    playback::pipeline::{
        AudioPipelineError,
        config::{AudioPipelineConfiguration, AudioTrackPipelineConfiguration},
        stage::{
            AudioPipelineSamples, AudioPipelineStageCommandOutcome, AudioTrackPipelineStage, channel_converter::stage::{
                AudioPipelineChannelConverterStage, AudioPipelineChannelConverterStagePosition,
            }, decoder::{AudioDecoder, stage::AudioPipelineDecoderStage}, resampler::{AudioResampler, stage::AudioPipelineResamplerStage}
        },
        thread::AudioPipelineThreadCommand,
    },
    track::models::Track,
};

pub struct AudioTrackPipeline {
    pub status: AudioTrackPipelineStatus,
    stages: Vec<AudioTrackPipelineStage<AudioTrackPipelineConfiguration>>,

    configuration: AudioTrackPipelineConfiguration,
    frames_delivered: u64,
}

pub enum AudioTrackPipelineStatus {
    Ready,
    Ongoing,
    Finished,
}

impl AudioTrackPipeline {
    #[instrument(skip_all)]
    pub fn build(
        track: Track,
        pipeline_configuration: AudioPipelineConfiguration,
    ) -> Result<Self, AudioPipelineError> {
        let configuration = AudioTrackPipelineConfiguration::new(pipeline_configuration, track);

        let decoder = AudioDecoder::build(&configuration.track)?;

        let resampler = AudioResampler::build(
            configuration.track.sample_rate as u32,
            configuration.track.channels as u16,
            configuration.output.sample_rate,
            configuration.output.channels,
        )?;

        let audio_decoder_stage = AudioPipelineDecoderStage::new(decoder);

        let audio_resampler_stage = AudioPipelineResamplerStage::new(resampler);

        let stages = vec![
            AudioTrackPipelineStage::Source(Box::new(audio_decoder_stage)),
            AudioTrackPipelineStage::Process(Box::new(AudioPipelineChannelConverterStage::new(
                AudioPipelineChannelConverterStagePosition::BeforeResampling,
            ))),
            AudioTrackPipelineStage::Process(Box::new(audio_resampler_stage)),
            AudioTrackPipelineStage::Process(Box::new(AudioPipelineChannelConverterStage::new(
                AudioPipelineChannelConverterStagePosition::AfterResampling,
            ))),
        ];

        Ok(Self {
            configuration,

            status: AudioTrackPipelineStatus::Ready,

            stages,

            frames_delivered: 0,
        })
    }

    pub fn handle_command(
        &mut self,
        command: &AudioPipelineThreadCommand,
    ) -> Result<(), AudioPipelineError> {
        let mut outcomes = Vec::new();

        for stage in &mut self.stages {
            match stage {
                AudioTrackPipelineStage::Process(stage) => {
                    outcomes.push(stage.handle_command(&self.configuration, command)?);
                }
                AudioTrackPipelineStage::Source(stage) => {
                    outcomes.push(stage.handle_command(&self.configuration, command)?);
                }
            }
        }

        for outcome in outcomes {
            let Some(outcome) = outcome else {
                continue;
            };
            
            match outcome {
                AudioPipelineStageCommandOutcome::SeekedTo(new_timestamp) => {
                    self.frames_delivered = new_timestamp;
                }
            }
        }

        Ok(())
    }

    #[instrument(
        skip(self), 
        level = "debug",
        fields(
            track = self.configuration.track.file_path
                .rsplit_once('/')
                .map(|(_, file_name)| file_name )
                .unwrap_or(self.configuration.track.file_path.as_ref())
        )
    )]
    pub fn produce_samples(&mut self) -> Result<AudioPipelineSamples, AudioPipelineError> {
        if matches!(self.status, AudioTrackPipelineStatus::Finished) {
            return Err(AudioPipelineError::AudioTrackPipelineFinished);
        }

        let mut samples = AudioPipelineSamples::Chunk(vec![]);

        for stage in &mut self.stages {
            match stage {
                AudioTrackPipelineStage::Source(stage) => {
                    if stage.is_enabled(&self.configuration) {
                        samples = stage.process_stage(&self.configuration)?;
                    };
                }
                AudioTrackPipelineStage::Process(stage) => {
                    if stage.is_enabled(&self.configuration) {
                        samples = stage.process_stage(&self.configuration, samples)?;
                    }
                }
            };
        }

        self.frames_delivered += samples.len() as u64 / self.configuration.output.channels as u64;

        trace!(
            frames_delivered = samples.len() / self.configuration.output.channels as usize,
            total_frames_delivered = self.frames_delivered,
        );

        if matches!(samples, AudioPipelineSamples::End(_)) {
            if self.frames_delivered > self.configuration.track.frames as u64 {
                warn!("The audio track pipeline delivered more frames than expected");
            }

            self.status = AudioTrackPipelineStatus::Finished;
        }

        Ok(samples)
    }
}
