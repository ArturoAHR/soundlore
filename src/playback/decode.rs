use std::{path::PathBuf, sync::mpsc::Receiver};

pub struct AudioDecoder {
    pub command_receiver: Receiver<AudioDecoderCommand>,
    pub current_track: PathBuf,
    pub current_position: f32,
}

pub enum AudioDecoderCommand {
    Play(Option<PathBuf>),
    Stop,
    Seek(f32),
}
