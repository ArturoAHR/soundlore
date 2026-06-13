use std::time::Duration;

use cpal::{
    DeviceId, default_host,
    traits::{DeviceTrait, HostTrait},
};
use iced::{
    futures::{SinkExt, stream::Stream},
    stream,
};
use tracing::{error, instrument};

use crate::{app::Message, playback::Event};

#[instrument]
pub fn watch_default_device() -> impl Stream<Item = Message> {
    stream::channel(10, async |mut output| {
        let mut current_device_id = (async || {
            loop {
                let Some(device_id) = get_default_output_device_id().await else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                };

                return device_id;
            }
        })()
        .await;

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let Some(new_device_id) = get_default_output_device_id().await else {
                continue;
            };

            if current_device_id != new_device_id {
                if let Err(error) = output
                    .send(Message::Playback(Event::PendingOutputDeviceChange))
                    .await
                {
                    error!("Failed to send event: {error}");
                }

                current_device_id = new_device_id;
            }
        }
    })
}

async fn get_default_output_device_id() -> Option<DeviceId> {
    tokio::task::spawn_blocking(move || {
        let host = default_host();

        let device = host.default_output_device()?;

        device.id().ok()
    })
    .await
    .ok()?
}
