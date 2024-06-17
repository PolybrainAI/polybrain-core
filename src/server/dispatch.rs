use serde::Serialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio_util::codec::Framed;
use uuid::Uuid;
use crate::server::{auth::fetch_user_credentials, codec::{send_error, send_message, wait_for_message, FramedSocket, MessageCodec}, error::AuthenticationError, types::{ApiCredentials, SessionStartResponse}};

use super::{error::SocketError, types::{SessionStartRequest, UserPrompt}};
use std::{error::Error, io::Result};

async fn start_execution_loop(mut socket: TcpStream) -> Result<()>{
    println!("Spawned new task for socket: {:?}", socket);

    let mut framed: FramedSocket = Framed::new(socket, MessageCodec);

    println!("waiting for incoming message...");
    let incoming: SessionStartRequest = wait_for_message(&mut framed).await?;

    let credentials: ApiCredentials;

    match fetch_user_credentials(&incoming.user_token){
        Ok(c) => {credentials = c},
        Err(message) => {
            send_error(&mut framed, AuthenticationError{ message }).await?;
            return Ok(())
        }
    }

    let session_id = Uuid::new_v4().to_string();
    println!("staring session with id {session_id}");

    send_message(&mut framed, SessionStartResponse {session_id}).await?;


    


    Ok(())
}



/// Dispatches an incoming socket connection
pub async fn dispatch_incoming(socket: TcpStream){
    tokio::spawn(async move  {

        if let Err(err) = start_execution_loop(socket).await {
            eprintln!("tokio process errored: {}", err)
        }

    });
}