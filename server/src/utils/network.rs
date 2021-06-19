use crate::types::WsMessage;
use shared::{Data, Message};

use anyhow::{anyhow, Result};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::time::Instant;
use warp::ws::WebSocket;

/// Send client challenge message and waits for response
/// Time taken by client to respond to message is recorded.
/// If `profile_roundtrip_time` is true then the time measurement
/// starts before sending message, otherwise it starts after
/// message is sent.
pub(crate) async fn send_client_msg_with_profiling(
    write_half: &mut SplitSink<WebSocket, WsMessage>,
    read_half: &mut SplitStream<WebSocket>,
    bytes: &[u8],
    profile_roundtrip_time: bool,
) -> Result<(Message, u128)> {
    let instant: Instant;
    let msg = WsMessage::binary(bytes);

    if profile_roundtrip_time {
        instant = Instant::now();
        write_half.send(msg).await?;
    } else {
        write_half.send(msg).await?;
        instant = Instant::now();
    }

    let response = read_half
        .next()
        .await
        .ok_or_else(|| anyhow!("Can't read client response, the stream was closed"))?
        .map_err(|e| anyhow!("Error reading from stream: {:?}", e))?;

    let time_elapsed = instant.elapsed().as_millis();

    if !response.is_binary() {
        return Err(anyhow!(
            "Wrong message format, expected to be a binary data"
        ));
    }

    let msg = Message::decode(response.as_bytes())?;
    let client_error = match &msg {
        &Message::Data(ref data) => match data {
            Data::Error(e) => Some(anyhow!(format!("Client returned an error: {}", e))),
            _ => None,
        },
        _ => None,
    };

    if client_error.is_some() {
        Err(client_error.unwrap())
    } else {
        Ok((msg, time_elapsed))
    }
}
