use thiserror::Error;

use crate::playback::pipeline::{AudioPipelineStatus, thread::AudioPipelineThreadCommand};

#[derive(Error, Debug, Clone)]
pub enum AudioPipelineCommandReceiverError {
    #[error("Failed to receive audio pipeline command: {0}")]
    ReceiveFailed(#[from] std::sync::mpsc::RecvError),

    #[error("Failed attempt to receive audio pipeline command: {0}")]
    ReceiveAttemptFailed(#[from] std::sync::mpsc::TryRecvError),
}

pub type CommandReceiver = std::sync::mpsc::Receiver<AudioPipelineThreadCommand>;

pub struct AudioPipelineCommandReceiver {
    command_receiver: CommandReceiver,
}

impl AudioPipelineCommandReceiver {
    pub fn new(command_receiver: CommandReceiver) -> Self {
        Self { command_receiver }
    }

    pub fn receive(
        &self,
        status: &AudioPipelineStatus,
    ) -> Result<Option<AudioPipelineThreadCommand>, AudioPipelineCommandReceiverError> {
        if matches!(
            status,
            AudioPipelineStatus::Idle | AudioPipelineStatus::Paused
        ) {
            let command = self
                .command_receiver
                .recv()
                .map_err(AudioPipelineCommandReceiverError::ReceiveFailed)?;

            return Ok(Some(command));
        }

        match self.command_receiver.try_recv() {
            Ok(command) => Ok(Some(command)),
            Err(error) => match error {
                std::sync::mpsc::TryRecvError::Empty => Ok(None),
                std::sync::mpsc::TryRecvError::Disconnected => Err(error.into()),
            },
        }
    }
}
