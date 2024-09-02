use core::fmt;
use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use tokio::net::TcpStream;

use super::error::SocketError;

/// Waits for an incoming message of a certain type
pub async fn wait_for_message<T: DeserializeOwned + Serialize + fmt::Debug>(
    ws_stream: &mut WebSocketStream<&mut TcpStream>,
) -> Result<T, String> {
    let (_, mut read) = ws_stream.split();

    if let Some(message) = read.next().await {
        let message = message.map_err(|_| "Corrupted message in websocket".to_owned())?;

        if message.is_text() {
            let message_text = message
                .to_text()
                .map_err(|_| "Message is not UTF-8".to_owned())?;
            match serde_json::from_str(message_text) {
                Ok(model) => {
                    println!("got incoming message:\n{:?}", &model);
                    Ok(model)
                }
                Err(err) => {
                    println!("failed to deserialize incoming message:\n{:?}", err);
                    println!("the message was:\n{message_text}");
                    Err("Bad Request: Unable to deserialize incoming message".to_owned())
                }
            }
        } else {
            Err("incoming websocket message was not text".to_owned())
        }
    } else {
        Err("Connection closed".to_owned())
    }
}

/// Sends an outbound message
pub async fn send_message<T: Serialize>(
    ws_stream: &mut WebSocketStream<&mut TcpStream>,
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
    ws_stream: &mut WebSocketStream<&mut TcpStream>,
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
