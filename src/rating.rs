//! Ratings of players for various cricket skills
use serde::{Deserialize, Serialize};

/// All skill ratings grouped
#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerRating {
    pub batting: BatRating,
    pub bowling: BowlRating,
    pub fielding: FieldRating,
}

/// Ratings for batting
#[derive(Debug, Deserialize, Serialize)]
pub struct BatRating {}
/// Ratings for bowling
#[derive(Debug, Deserialize, Serialize)]
pub struct BowlRating {}
/// Ratings for fielding and wicket-keeping
#[derive(Debug, Deserialize, Serialize)]
pub struct FieldRating {}
