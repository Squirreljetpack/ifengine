use thiserror::Error;

#[derive(Debug, Error, std::hash::Hash, PartialEq, Eq, Clone)]
pub enum GameError {
    #[error("no page stack available")]
    NoStack,
    #[error("no page available")]
    NoPage,
    #[error("Game end")]
    End,
}

#[derive(Debug, Error, std::hash::Hash, PartialEq, Eq, Clone)]
pub enum SimEnd {
    #[error("{0}")]
    GameError(#[from] GameError),
    #[error("Tunnel Entrance")]
    Tunnel(String),
    #[error("Tunnel Exit")]
    TunnelExit,
    #[error("{0}")]
    Custom(String)
}