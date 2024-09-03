use std::{error::Error, fmt};

use crate::chain::agents::onpy_agent::CodeError;

#[derive(Debug)]
#[allow(dead_code)]
pub enum PolybrainError {
    InternalError(String),
    SocketError(String),
    BadRequest(String),
    CodeError(CodeError),
    Unreachable,
}

impl fmt::Display for PolybrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for PolybrainError {}

// pub trait SocketError {
//     fn serialize_string(&self) -> String
//     where
//         Self: Serialize,
//     {
//         let json_str = serde_json::to_string(&self).unwrap();
//         let v: Value = serde_json::from_str(&json_str).unwrap();
//         let mut map: HashMap<String, String> = serde_json::from_value(v).unwrap();
//         map.insert("error_type".to_owned(), Self::name());

//         serde_json::to_string_pretty(&map).unwrap()
//     }
//     fn name() -> String;
// }

// #[derive(Serialize)]
// pub struct AuthenticationError {
//     pub message: String,
// }
// impl SocketError for AuthenticationError {
//     fn name() -> String {
//         "AuthenticationError".to_string()
//     }
// }

// #[derive(Serialize, Debug)]
// pub struct RequestError {
//     pub message: String,
//     pub operation: String,
// }
// impl SocketError for RequestError {
//     fn name() -> String {
//         "RequestError".to_string()
//     }
// }

// #[derive(Serialize)]
// pub struct InternalError {
//     pub message: String,
// }
// impl SocketError for InternalError {
//     fn name() -> String {
//         "InternalError".to_string()
//     }
// }
