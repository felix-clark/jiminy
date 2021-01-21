//! Ratings of players for various cricket skills

/// All skill ratings grouped
#[derive(Debug)]
pub struct PlayerRating {
    pub batting: BatRating,
    pub bowling: BowlRating,
    pub fielding: FieldRating,
}

/// Ratings for batting
#[derive(Debug)]
pub struct BatRating {}
/// Ratings for bowling
#[derive(Debug)]
pub struct BowlRating {}
/// Ratings for fielding and wicket-keeping
#[derive(Debug)]
pub struct FieldRating {}
