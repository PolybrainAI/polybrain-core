use serde::{Deserialize, Serialize};

pub const ONSHAPE_API: &str = "https://cad.onshape.com/api/v6";
pub const OPENAI_API: &str = "https://api.openai.com/v1";

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStartRequest {
    pub user_token: String,
    pub onshape_document_id: String,
}

#[derive(Serialize)]
pub struct SessionStartResponse {
    pub session_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserPromptInitial {
    pub contents: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct UserInputResponse {
    pub response: String,
}

#[derive(Serialize, Debug)]
pub enum ServerResponseType {
    _Query,
    Info,
    Final,
}

#[derive(Serialize, Debug)]
pub struct ServerResponse {
    pub response_type: ServerResponseType,
    pub content: String,
}

#[derive(Debug)]
pub struct ApiCredentials {
    pub openai_token: String,
    pub onshape_access_key: String,
    pub onshape_secret_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDocumentCredentials {
    pub onshape_access: Option<String>,
    pub onshape_secret: Option<String>,
    pub open_ai_api: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDocument {
    pub user_id: String,
    pub email: String,
    pub credentials: UserDocumentCredentials,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub created_at: String,
    pub email: String,
    pub name: String,
    pub user_id: String,
    pub username: Option<String>,
    pub last_ip: String,
    pub last_login: String,
    pub given_name: Option<String>,
}
