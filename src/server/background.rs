
use futures::StreamExt;
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};
use tokio_tungstenite::WebSocketStream;

use crate::server::error::PolybrainError;

use super::types::{ServerResponse, SessionStartRequest, SessionStartResponse, UserPromptInitial};

#[allow(dead_code)] // NOTE: Remove after implementing
pub enum BackgroundRequest {
    GetSessionStart,
    RespondSessionStart(SessionStartResponse),
    GetInitialInput,
    UserQuery(String),
    SendOutput(ServerResponse),
    End(ServerResponse),
}

#[allow(dead_code)] // NOTE: Remove after implementing
pub enum BackgroundResponse {
    SessionStart(SessionStartRequest),
    InitialInput(UserPromptInitial),
    UserResponse(String),
    Ack,
}

pub struct BackgroundClient {
    tx: Sender<BackgroundRequest>,
    rx: Receiver<BackgroundResponse>,
}

impl BackgroundClient {
    pub fn connect(tx: Sender<BackgroundRequest>, rx: Receiver<BackgroundResponse>) -> Self {
        Self { tx, rx }
    }

    async fn wait_for_background_response(&mut self) -> Result<BackgroundResponse, PolybrainError> {
        let mut response = self.rx.recv().await;
        while response.is_none() {
            response = self.rx.recv().await
        }
        Ok(response.unwrap())
    }

    pub async fn send(
        &mut self,
        request: BackgroundRequest,
    ) -> Result<BackgroundResponse, PolybrainError> {
        self.tx
            .send(request)
            .await
            .map_err(|err| PolybrainError::InternalError(err.to_string()))?;
        self.wait_for_background_response().await
    }
}

pub struct BackgroundTask<'b> {
    rx: Receiver<BackgroundRequest>,
    tx: Sender<BackgroundResponse>,
    ws: WebSocketStream<&'b mut TcpStream>,
    alive: bool,
}

impl<'b> BackgroundTask<'b> {
    pub fn new(
        rx: Receiver<BackgroundRequest>,
        tx: Sender<BackgroundResponse>,
        ws: WebSocketStream<&'b mut TcpStream>,
    ) -> Self {
        Self {
            tx,
            rx,
            ws,
            alive: true,
        }
    }

    async fn get_session_start(&mut self) -> Result<BackgroundResponse, PolybrainError> {
        println!("Waiting for incoming session start...");

        let incoming = self.wait_ws_message().await?;

        let session_start_request: SessionStartRequest = serde_json::from_slice(&incoming)
            .map_err(|err| {
                PolybrainError::BadRequest(format!(
                    "Failed to parse SessionStartRequest:\n {:#?}",
                    err
                ))
            })?;

        Ok(BackgroundResponse::SessionStart(session_start_request))
    }

    /// Dispatches an incoming request to the corresponding handle
    async fn dispatch(
        &mut self,
        request: BackgroundRequest,
    ) -> Result<BackgroundResponse, PolybrainError> {
        match request {
            BackgroundRequest::GetSessionStart => self.get_session_start().await,
            _ => todo!(),
        }
    }

    /// Wait for incoming websocket message
    async fn wait_ws_message(&mut self) -> Result<Vec<u8>, PolybrainError> {
        let ws_stream = &mut self.ws;

        let (_, mut read) = ws_stream.split();

        if let Some(message) = read.next().await {
            let message = message.map_err(|_| {
                PolybrainError::SocketError("Corrupted message in websocket".to_owned())
            })?;

            if message.is_text() {
                let message_text = message
                    .to_text()
                    .map_err(|_| PolybrainError::SocketError("Message is not UTF-8".to_owned()))?;
                Ok(message_text.as_bytes().to_vec())
            } else {
                Err(PolybrainError::SocketError(
                    "Message should be type text".to_owned(),
                ))
            }
        } else {
            Err(PolybrainError::SocketError("Connection closed".to_owned()))
        }
    }

    /// Begin background loop
    pub async fn begin(mut self) -> Result<(), PolybrainError> {
        while self.alive {
            let mut request = self.rx.recv().await;
            while request.is_none() {
                request = self.rx.recv().await;
            }
            let response = self.dispatch(request.unwrap()).await?;
            self.tx
                .send(response)
                .await
                .map_err(|err| PolybrainError::InternalError(err.to_string()))?;
        }

        Ok(())
    }
}
