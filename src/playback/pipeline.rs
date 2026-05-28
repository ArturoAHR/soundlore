use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use rtrb::Producer;
use symphonia::core::formats::TrackType::Audio;

pub struct AudioPipeline {
    pub command_receiver: Receiver<AudioPipelineCommand>,
    pub configuration: Option<AudioPipelineConfiguration>,
}

pub struct AudioPipelineConfiguration {
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_buffer_producer: Producer<f32>,
}

pub enum AudioPipelineCommand {
    Play(Option<PathBuf>),
    Pause,
    Stop,
    Seek(f32),
    ChangeConfiguration {
        sample_rate: u32,
        channels: u16,
        producer: Producer<f32>,
    },
    Exit,
}

impl AudioPipeline {
    pub fn new(command_receiver: Receiver<AudioPipelineCommand>) -> Self {
        Self {
            command_receiver,
            configuration: None,
        }
    }
}

pub fn spawn_audio_pipeline_thread() -> Sender<AudioPipelineCommand> {
    let (command_sender, command_receiver) = mpsc::channel();

    let audio_pipeline = AudioPipeline::new(command_receiver);

    std::thread::spawn(move || loop {
        if let Ok(pipeline_command) = audio_pipeline.command_receiver.try_recv() {
            match pipeline_command {
                AudioPipelineCommand::Exit => break,
                AudioPipelineCommand::Play(_) => todo!(),
                AudioPipelineCommand::ChangeConfiguration {
                    sample_rate: _,
                    channels: _,
                    producer: _,
                } => todo!(),
                AudioPipelineCommand::Stop => todo!(),
                AudioPipelineCommand::Pause => todo!(),
                AudioPipelineCommand::Seek(_) => todo!(),
            }
        }
    });

    command_sender
}
