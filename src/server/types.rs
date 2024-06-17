use serde::{Deserialize, Serialize};

enum SocketMessage {
    SessionStartRequest(SessionStartRequest),

}

#[derive(Deserialize, Serialize)]
pub struct SessionStartRequest {
    pub user_token: String
}

#[derive(Serialize)]
pub struct SessionStartResponse {
    pub session_id: String
}

pub struct UserPrompt {
    pub contents: String
}

pub struct ApiCredentials {
    pub openai_token: String,
    pub onshape_access_key: String,
    pub onshape_secret_key: String,
}

