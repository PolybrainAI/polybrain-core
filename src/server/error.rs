use serde::Serialize;

pub trait SocketError {
    fn serialize_string(&self) -> String
    where
        Self: Serialize,
    {
        serde_json::to_string_pretty(&self).expect("Failed to serialize error type")
    }
    fn name() -> String;
}

#[derive(Serialize)]
pub struct AuthenticationError {
    pub message: String,
}
impl SocketError for AuthenticationError {
    fn name() -> String {
        "AuthenticationError".to_string()
    }
}

#[derive(Serialize, Debug)]
pub struct RequestError {
    pub message: String,
    pub operation: String,
}
impl SocketError for RequestError {
    fn name() -> String {
        "RequestError".to_string()
    }
}

#[derive(Serialize)]
pub struct InternalError {
    pub message: String,
}
impl SocketError for InternalError {
    fn name() -> String {
        "InternalError".to_string()
    }
}
