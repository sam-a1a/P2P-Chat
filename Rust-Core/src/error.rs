use thiserror::Error;

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("transport: {0}")]
    Transport(String),

    #[error("behaviour: {0}")]
    Behaviour(String),

    #[error("identity: {0}")]
    Identity(String),

    #[error("crypto: {0}")]
    Crypto(String),

    #[error("serialization: {0}")]
    Serialization(String),

    #[error("storage: {0}")]
    Storage(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("command channel closed")]
    ChannelClosed,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, P2pError>;