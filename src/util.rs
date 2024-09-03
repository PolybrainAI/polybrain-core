use std::{error::Error, fmt};

use crate::chain::agents::onpy_agent::CodeError;

/// Gets a dotenv variable. Panics if unbound
pub fn get_dotenv(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set in .env"))
}

#[derive(Debug)]
pub enum PolybrainError {
    InternalError(String),
    SocketError(String),
    BadRequest(String),
    CodeError(CodeError),
    NotImplemented,
    Unreachable,
    Other,
}

impl fmt::Display for PolybrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for PolybrainError {}
