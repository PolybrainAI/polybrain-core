use crate::{
    chain::chain::enter_chain,
    server::{
        auth::fetch_user_credentials,
        codec::{send_error, send_message, wait_for_message, FramedSocket, MessageCodec},
        error::{AuthenticationError, InternalError},
        types::{ApiCredentials, SessionStartResponse, UserInputQuery, UserPromptInitial},
    },
};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_util::codec::Framed;
use uuid::Uuid;

use super::types::{ServerResponse, SessionStartRequest, UserInputResponse};
use std::io::Result;

async fn query_input_callback(frame_mutex: &Mutex<FramedSocket>, input: &str) -> Result<String> {
    let mut frame = frame_mutex.lock().await;

    send_message(
        &mut frame,
        UserInputQuery {
            query: input.to_owned(),
        },
    )
    .await?;

    let response: UserInputResponse = wait_for_message(&mut frame).await?;

    Ok(response.response)
}

async fn send_output_callback(
    frame_mutex: &Mutex<FramedSocket>,
    output: ServerResponse,
) -> Result<()> {
    let mut frame = frame_mutex.lock().await;

    send_message(&mut frame, output).await?;

    Ok(())
}

async fn start_execution_loop(socket: TcpStream) -> Result<()> {
    println!("Spawned new task for socket: {:?}", socket);

    let mut framed: FramedSocket = Framed::new(socket, MessageCodec);

    println!("waiting for incoming message...");
    let incoming: SessionStartRequest = wait_for_message(&mut framed).await?;

    let credentials: ApiCredentials = match fetch_user_credentials(&incoming.user_token).await {
        Ok(c) => c,
        Err(message) => {
            send_error(&mut framed, AuthenticationError { message }).await?;
            return Ok(());
        }
    };

    _ = credentials; // TODO: remove once credentials are used

    let session_id = Uuid::new_v4().to_string();
    println!("staring session with id {session_id}");

    send_message(&mut framed, SessionStartResponse { session_id }).await?;

    let initial_input: UserPromptInitial = wait_for_message(&mut framed).await?;

    let frame_mutex = Mutex::new(framed);

    if let Err(err) = enter_chain(
        &initial_input.contents,
        |input: &str| Box::pin(query_input_callback(&frame_mutex, input)),
        |output: ServerResponse| Box::pin(send_output_callback(&frame_mutex, output)),
    )
    .await
    {
        println!("LLM Chain Crashed with error: {}", err);
        let mut frame = frame_mutex.lock().await;
        send_error(
            &mut frame,
            InternalError {
                message: "LLM Chain experienced an unrecoverable error".to_owned(),
            },
        )
        .await?;
    }

    Ok(())
}

/// Dispatches an incoming socket connection
pub async fn dispatch_incoming(socket: TcpStream) {
    tokio::spawn(async move {
        if let Err(err) = start_execution_loop(socket).await {
            eprintln!("tokio process errored: {}", err)
        }
    });
}
