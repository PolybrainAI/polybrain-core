use bytes::{BufMut, BytesMut};
use futures::{SinkExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::io;

use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

use super::error::{RequestError, SocketError};

pub struct MessageCodec;

impl Decoder for MessageCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if we have a complete line (terminated by \r\n)
        if let Some(pos) = src.windows(2).position(|window| window == b"\r\n") {
            // Remove the line from the buffer
            let mut line = src.split_to(pos + 2);
            // Remove the \r\n
            _ = line.split_off(pos);
            // Convert the bytes to a string and return it
            let line = String::from_utf8(line.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(Some(line))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<&str> for MessageCodec {
    type Error = io::Error;

    fn encode(&mut self, item: &str, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Append the item and \r\n to the buffer
        dst.put(item.as_bytes());
        dst.put(&b"\r\n"[..]);
        Ok(())
    }
}

pub type FramedSocket = Framed<TcpStream, MessageCodec>;

/// Waits for an incoming message of a certain type
pub async fn wait_for_message<T: DeserializeOwned + Serialize>(
    socket: &mut FramedSocket,
) -> io::Result<T> {
    let mut next_message = socket.try_next().await?;

    while next_message.is_none() {
        next_message = socket.try_next().await?;
    }
    match serde_json::from_str(&next_message.clone().unwrap()) {
        Ok(model) => {
            println!(
                "got incoming message:\n{}",
                serde_json::to_string_pretty(&model)?
            );
            Ok(model)
        }
        Err(err) => {
            println!(
                "Unable to deserialize the incoming message:\n{}",
                next_message.unwrap()
            );
            send_error(
                socket,
                RequestError {
                    message: format!("Bad Request Format: {}", err),
                    operation: "Deserialize Request".to_owned(),
                },
            )
            .await?;
            Err(err.into())
        }
    }
}

/// Sends an outbound message
pub async fn send_message<T: Serialize>(socket: &mut FramedSocket, payload: T) -> io::Result<()> {
    let payload_string = serde_json::to_string_pretty(&payload)?;
    println!("sending message:\n{payload_string}");
    socket
        .send(serde_json::to_string_pretty(&payload)?.as_str())
        .await?;
    Ok(())
}

/// Sends an error response
pub async fn send_error<T>(socket: &mut FramedSocket, error: T) -> io::Result<()>
where
    T: SocketError + Serialize,
{
    println!("sending error:");
    send_message(socket, error).await?;
    Ok(())
}
