use core::fmt;
use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use tokio::net::TcpStream;

use super::error::SocketError;

/// Waits for an incoming message of a certain type
pub async fn wait_for_message<T: DeserializeOwned + Serialize + fmt::Debug>(
    ws_stream: &mut WebSocketStream<TcpStream>,
) -> Result<T, Box<dyn Error>> {
    let (_, mut read) = ws_stream.split();

    if let Some(message) = read.next().await {
        let message = message.expect("websocket has corrupted message");

        if message.is_text() {
            let message_text = message.to_text()?;
            match serde_json::from_str(message_text) {
                Ok(model) => {
                    println!("got incoming message:\n{:?}", &model);
                    return Ok(model);
                }
                Err(err) => {
                    println!("failed to deserialize incoming message:\n{:?}", err);
                    println!("the message was:\n{message_text}");
                    return Err(Box::new(err));
                }
            }
        } else {
            println!("incoming websocket message was not text!");
            return Err(Box::new(tungstenite::Error::Utf8));
        }
    } else {
        return Err(Box::new(tungstenite::Error::ConnectionClosed));
    };
}

/// Sends an outbound message
pub async fn send_message<T: Serialize>(
    ws_stream: &mut WebSocketStream<TcpStream>,
    payload: T,
) -> Result<(), Box<dyn Error>> {
    let (mut write, _) = ws_stream.split();

    let payload_string = serde_json::to_string_pretty(&payload)?;
    println!("sending output:\n{}", payload_string);
    write.send(Message::text(payload_string)).await?;

    Ok(())
}

/// Sends an error response
pub async fn send_error<T>(
    ws_stream: &mut WebSocketStream<TcpStream>,
    error: T,
) -> Result<(), Box<dyn Error>>
where
    T: SocketError + Serialize,
{
    let payload_string = &error.serialize_string();
    println!("sending error:\n{}", payload_string);

    let (mut write, _) = ws_stream.split();
    write.send(Message::text(payload_string)).await?;

    Ok(())
}
