use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::server::{error::PolybrainError, types::UserQueryResponse};

use super::types::{ServerResponse, ServerResponseType, SessionStartRequest, SessionStartResponse, UserPromptInitial};

pub enum BackgroundRequest {
    GetSessionStart,
    RespondSessionStart(SessionStartResponse),
    GetInitialInput,
    UserQuery(String),
    SendOutput(ServerResponse),
    End(ServerResponse),
}

pub enum BackgroundResponse {
    SessionStart(SessionStartRequest),
    InitialInput(UserPromptInitial),
    UserResponse(UserQueryResponse),
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

    async fn get_initial_input(&mut self) -> Result<BackgroundResponse, PolybrainError> {
        println!("Waiting for initial input...");

        let incoming = self.wait_ws_message().await?;

        let initial_input: UserPromptInitial = serde_json::from_slice(&incoming)
            .map_err(|err| {
                PolybrainError::BadRequest(format!(
                    "Failed to parse UserPromptInitial:\n {:#?}",
                    err
                ))
            })?;

        Ok(BackgroundResponse::InitialInput(initial_input))
    }

    async fn respond_session_start(&mut self, response: SessionStartResponse) -> Result<BackgroundResponse, PolybrainError>{
        self.send_ws_message(&response).await?;
        Ok(BackgroundResponse::Ack)
    }

    async fn get_user_query(&mut self, query: String) -> Result<BackgroundResponse, PolybrainError>{

        let payload = ServerResponse{
            response_type: ServerResponseType::Query,
            content: query
        };

        self.send_ws_message(&payload).await?;

        let response_raw = self.wait_ws_message().await?;

        let response: UserQueryResponse = serde_json::from_slice(&response_raw).map_err(|err| PolybrainError::BadRequest(format!("Unable to deserialize query response:\n{}", err)))?;

        Ok(BackgroundResponse::UserResponse(response))

    }

    async fn send_output(&mut self, output: ServerResponse) -> Result<BackgroundResponse, PolybrainError>{
        self.send_ws_message(&output).await?;
        Ok(BackgroundResponse::Ack)
    }

    async fn send_end(&mut self, output: ServerResponse) -> Result<BackgroundResponse, PolybrainError>{
        self.send_ws_message(&output).await?;
        self.alive = false;
        Ok(BackgroundResponse::Ack)
    }

    /// Dispatches an incoming request to the corresponding handle
    async fn dispatch(
        &mut self,
        request: BackgroundRequest,
    ) -> Result<BackgroundResponse, PolybrainError> {
        match request {
            BackgroundRequest::GetSessionStart => self.get_session_start().await,
            BackgroundRequest::GetInitialInput => self.get_initial_input().await,
            BackgroundRequest::RespondSessionStart(r) => self.respond_session_start(r).await,
            BackgroundRequest::UserQuery(q) => self.get_user_query(q).await,
            BackgroundRequest::SendOutput(o) => self.send_output(o).await,
            BackgroundRequest::End(r) => self.send_end(r).await
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

    /// Send a message to the websocket client
    /// 
    /// Args:
    /// * `payload` - The serializable payload to send
    async fn send_ws_message<T: Serialize>(&mut self, payload: &T) -> Result<(), PolybrainError> {
        let ws_stream = &mut self.ws;
        let (mut write, _) = ws_stream.split();

        let payload_str = serde_json::to_string_pretty(payload).expect("Serde could not serialize payload");
        println!("Sending output: {}", payload_str);

        write.send(Message::Text(payload_str)).await.map_err(|err| PolybrainError::SocketError(err.to_string()))?;

        Ok(())
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
