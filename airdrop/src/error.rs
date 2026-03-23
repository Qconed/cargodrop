use thiserror::Error;

#[derive(Error, Debug)]
pub enum AirdropError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Transfer error: {0}")]
    Transfer(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("QUIC error: {0}")]
    Quic(String),
}

pub type Result<T> = std::result::Result<T, AirdropError>;