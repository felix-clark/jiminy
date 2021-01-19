//! Teams of players
use crate::rating::PlayerRating;

pub struct Team {
    pub players: Vec<Player>,
}

pub struct Player {
    name: String,
    rating: PlayerRating,
}
