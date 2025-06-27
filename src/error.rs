//! Error module: Defines custom error types for the P2P walkie-talkie application.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalkieTalkieError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for WalkieTalkieError {
    fn from(e: std::io::Error) -> Self {
        WalkieTalkieError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for WalkieTalkieError {
    fn from(e: serde_json::Error) -> Self {
        WalkieTalkieError::Serialization(e.to_string())
    }
}
