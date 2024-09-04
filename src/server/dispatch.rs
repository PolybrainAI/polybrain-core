use crate::{
    chain::chain_entry::enter_chain,
    server::background::{BackgroundClient, BackgroundTask},
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::mpsc::{self},
};
use tokio_tungstenite::accept_async;

use super::background::{BackgroundRequest, BackgroundResponse};

// async fn query_input_callback(
//     ws_mutex: &Mutex<WebSocketStream<&mut TcpStream>>,
//     input: String,
// ) -> Result<String, Box<dyn Error>> {
//     let mut ws_stream = ws_mutex.lock().await;

//     send_message(
//         &mut ws_stream,
//         ServerResponse {
//             response_type: ServerResponseType::Query,
//             content: input.to_owned(),
//         },
//     )
//     .await?;

//     let incoming: UserQueryResponse = wait_for_message(&mut ws_stream).await?;
//     Ok(incoming.response)
// }

// async fn send_output_callback(
//     ws_mutex: &Mutex<WebSocketStream<&mut TcpStream>>,
//     output: ServerResponse,
// ) -> Result<(), Box<dyn Error>> {
//     let mut ws_stream = ws_mutex.lock().await;

//     send_message(&mut ws_stream, output).await?;

//     Ok(())
// }

fn build_html_response() -> String {
    let html_error = include_str!("html_error.html");
    format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: text/html; charset=UTF-8\r\n\
        Content-Length: {len}\r\n\
        Connection: close\r\n\r\n\
        {content}",
        len = html_error.len(),
        content = html_error
    )
}

async fn process(mut socket: TcpStream) {
    println!("converting incoming tcp to websocket: {:?}", socket);
    let ws_stream = match accept_async(&mut socket).await {
        Ok(stream) => stream,
        Err(_) => {
            socket
                .write_all(build_html_response().as_bytes())
                .await
                .expect("Failed to write to socket");
            return;
        }
    };

    let (request_tx, request_rx) = mpsc::channel::<BackgroundRequest>(128);
    let (response_tx, response_rx) = mpsc::channel::<BackgroundResponse>(128);

    let background_task = BackgroundTask::new(request_rx, response_tx, ws_stream);
    let bridge = BackgroundClient::connect(request_tx, response_rx);

    tokio::spawn(async move { enter_chain(bridge).await });

    // Background listener
    let output = background_task.begin().await;

    match output {
        Ok(_) => {
            println!("Successfully completed chain. Closing connection.")
        }
        Err(err) => {
            println!("ERROR: {err}");
        }
    }
}

/// Dispatches an incoming socket connection
pub async fn dispatch_incoming(socket: TcpStream) {
    tokio::spawn(async move {
        process(socket).await;
    });
}
