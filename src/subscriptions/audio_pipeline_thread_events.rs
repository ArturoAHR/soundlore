use iced::futures::{SinkExt, StreamExt, stream::Stream};
use iced::stream;

use tracing::{error, instrument};

use crate::{app, playback};

#[instrument]
pub fn audio_pipeline_thread_events() -> impl Stream<Item = app::Message> {
    stream::channel(100, async |mut output| {
        let (audio_pipeline_event_sender, mut audio_pipeline_event_receiver) =
            iced::futures::channel::mpsc::unbounded();

        if let Err(error) = output
            .send(app::Message::AudioPipelineEventChannelReady(
                audio_pipeline_event_sender,
            ))
            .await
        {
            error!("Failed to send audio pipeline event channel sender: {error}");
        }

        while let Some(event) = audio_pipeline_event_receiver.next().await {
            if let Err(error) = output
                .send(app::Message::Playback(
                    playback::Message::AudioPipelineEvent(event),
                ))
                .await
            {
                error!("Failed to send audio pipeline event channel sender: {error}");
            }
        }
    })
}
