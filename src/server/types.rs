use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SessionStartRequest {
    pub user_token: String,
}

#[derive(Serialize)]
pub struct SessionStartResponse {
    pub session_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserPromptInitial {
    pub contents: String,
}

#[derive(Serialize)]
pub struct UserInputQuery {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserInputResponse {
    pub response: String,
}

#[derive(Serialize)]
pub enum ServerResponseType {
    Info,
    Final,
}

#[derive(Serialize)]
pub struct ServerResponse {
    pub response_type: ServerResponseType,
    pub content: String,
}

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