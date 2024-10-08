use crate::{
    chain::chain_entry::enter_chain,
    server::{
        auth::fetch_user_credentials,
        codec::{send_error, send_message, wait_for_message},
        error::{AuthenticationError, InternalError},
        types::{ApiCredentials, SessionStartResponse, UserPromptInitial},
    },
};
use std::error::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};
use tokio_tungstenite::{accept_async, WebSocketStream};

use uuid::Uuid;

use super::types::{ServerResponse, ServerResponseType, SessionStartRequest, UserInputResponse};

async fn query_input_callback(
    ws_mutex: &Mutex<WebSocketStream<&mut TcpStream>>,
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
    ws_mutex: &Mutex<WebSocketStream<&mut TcpStream>>,
    output: ServerResponse,
) -> Result<(), Box<dyn Error>> {
    let mut ws_stream = ws_mutex.lock().await;

    send_message(&mut ws_stream, output).await?;

    Ok(())
}

async fn start_execution_loop(
    mut ws_stream: WebSocketStream<&mut TcpStream>,
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
        incoming.onshape_document_id,
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

async fn process(mut socket: TcpStream) {
    println!("converting incoming tcp to websocket: {:?}", socket);
    let ws_stream = match accept_async(&mut socket).await {
        Ok(stream) => stream,
        Err(_) => {
            socket
                .write_all(
                    "HTTP/1.1 200 OK\r\n\
                        Content-Type: text/html; charset=UTF-8\r\n\
                        Content-Length: 128\r\n\
                        Connection: close\r\n\r\n\
                        <!DOCTYPE html><html lang=\"en\"><head></head><body>Expected websocket. If you are lost, go to https://polybrain.xyz</body></html>".as_bytes(),
                )
                .await
                .expect("Failed to write to socket");
            return;
        }
    };

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
