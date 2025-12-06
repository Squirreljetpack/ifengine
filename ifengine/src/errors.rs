use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, std::hash::Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum GameError {
    #[error("NoStack")]
    NoStack,
    #[error("NoPage")]
    NoPage,
    #[error("GameEnd")]
    End,
}

#[derive(Debug, Error, std::hash::Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SimEnd {
    #[error("⟨{0}⟩")]
    GameError(#[from] GameError),
    #[error("{0}")]
    Tunnel(String),
    #[error("⟨Exit⟩")]
    TunnelExit,
    #[error("⟨{0}⟩")]
    Custom(String)
}