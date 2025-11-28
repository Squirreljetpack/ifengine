use thiserror::Error;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("no page stack available")]
    NoStack,
    #[error("no page available")]
    NoPage,
    #[error("Game end")]
    End,
}