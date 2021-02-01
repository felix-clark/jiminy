//! Library-specific error type
use crate::player::PlayerId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not find player with ID {0}")]
    PlayerNotFound(PlayerId),
    #[error("Duplicate player ID: {0}")]
    DuplicatePlayerId(PlayerId),
    #[error("Match is complete")]
    MatchComplete,
    #[error("Object not available: {0}")]
    MissingData(String),
}

pub type Result<T> = std::result::Result<T, Error>;
