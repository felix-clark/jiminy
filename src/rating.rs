//! Ratings of players for various cricket skills

/// All skill ratings grouped
pub struct PlayerRating {
    pub batting: BatRating,
    pub bowling: BowlRating,
    pub fielding: FieldRating,
}

/// Ratings for batting
pub struct BatRating {}
/// Ratings for bowling
pub struct BowlRating {}
/// Ratings for fielding and wicket-keeping
pub struct FieldRating {}
