use crate::{
    chain::chain::enter_chain,
    server::{
        auth::fetch_user_credentials,
        codec::{send_error, send_message, wait_for_message},
        error::{AuthenticationError, InternalError},
        types::{ApiCredentials, SessionStartResponse, UserPromptInitial},
    },
};
use std::error::Error;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{accept_async, WebSocketStream};

use uuid::Uuid;

use super::types::{ServerResponse, ServerResponseType, SessionStartRequest, UserInputResponse};

async fn query_input_callback(
    ws_mutex: &Mutex<WebSocketStream<TcpStream>>,
    input: String,
) -> Result<String, Box<dyn Error>> {
    let mut ws_stream = ws_mutex.lock().await;

    send_message(
        &mut ws_stream,
        ServerResponse {
            response_type: ServerResponseType::Query,
            content: input.to_owned(),
        },
    )
    .await?;

    let incoming: UserInputResponse = wait_for_message(&mut ws_stream).await?;
    Ok(incoming.response)
}

async fn send_output_callback(
    ws_mutex: &Mutex<WebSocketStream<TcpStream>>,
    output: ServerResponse,
) -> Result<(), Box<dyn Error>> {
    let mut ws_stream = ws_mutex.lock().await;

    send_message(&mut ws_stream, output).await?;

    Ok(())
}

async fn start_execution_loop(
    mut ws_stream: WebSocketStream<TcpStream>,
) -> Result<(), Box<dyn Error>> {
    println!("Spawned new task for socket");

    println!("waiting for incoming message...");
    let incoming: SessionStartRequest = wait_for_message(&mut ws_stream).await?;

    let credentials: ApiCredentials = match fetch_user_credentials(&incoming.user_token).await {
        Ok(c) => c,
        Err(message) => {
            send_error(&mut ws_stream, AuthenticationError { message }).await?;
            return Ok(());
        }
    };

    _ = credentials; // TODO: remove once credentials are used

    let session_id = Uuid::new_v4().to_string();
    println!("staring session with id {session_id}");

    send_message(&mut ws_stream, SessionStartResponse { session_id }).await?;

    let initial_input: UserPromptInitial = wait_for_message(&mut ws_stream).await?;

    let stream_mutex = Mutex::new(ws_stream);

    if let Err(err) = enter_chain(
        &initial_input.contents,
        credentials,
        |input: String| Box::pin(query_input_callback(&stream_mutex, input)),
        |output: ServerResponse| Box::pin(send_output_callback(&stream_mutex, output)),
    )
    .await
    {
        println!("LLM Chain Crashed with error: {}", err);
        let mut frame = stream_mutex.lock().await;
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

async fn process(socket: TcpStream) {
    println!("converting incoming tcp to websocket: {:?}", socket);
    let ws_stream = accept_async(socket)
        .await
        .expect("Failed to convert socket stream to ws");

    if let Err(err) = start_execution_loop(ws_stream).await {
        eprintln!("tokio process errored: {}", err)
    };
}

/// Dispatches an incoming socket connection
pub async fn dispatch_incoming(socket: TcpStream) {
    tokio::spawn(async move {
        process(socket).await;
    });
}
