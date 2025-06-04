use thiserror::Error;

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Error types for network operations
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Remote error: {0}")]
    RemoteError(String),

    #[error("Timeout error")]
    TimeoutError,

    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),

    #[error("libp2p error: {0}")]
    Libp2pError(String),
}

impl From<libp2p::TransportError<std::io::Error>> for NetworkError {
    fn from(err: libp2p::TransportError<std::io::Error>) -> Self {
        NetworkError::ConnectionError(err.to_string())
    }
}

impl From<libp2p::swarm::DialError> for NetworkError {
    fn from(err: libp2p::swarm::DialError) -> Self {
        NetworkError::ConnectionError(err.to_string())
    }
}

impl From<libp2p::request_response::OutboundFailure> for NetworkError {
    fn from(err: libp2p::request_response::OutboundFailure) -> Self {
        match err {
            libp2p::request_response::OutboundFailure::ConnectionClosed => {
                NetworkError::ConnectionError("Connection closed".into())
            }
            libp2p::request_response::OutboundFailure::Timeout => NetworkError::TimeoutError,
            _ => NetworkError::RequestFailed(format!("{:?}", err)),
        }
    }
}
