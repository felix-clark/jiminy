//! Teams of players
use crate::rating::PlayerRating;

#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub rating: PlayerRating,
}
