//! The interface and implementations for the cricket model(s)
use crate::game::{DeliveryOutcome, GameSnapshot};
use rand::Rng;
//use serde::{Deserialize, Serialize};

pub mod null;
pub use null::{NullModel, PlayerRatingNull};
pub mod naive_stats;
pub use naive_stats::{NaiveStatsModel, PlayerRatingNaiveStats};

pub trait PlayerRating {}

pub trait Model<R>
where
    R: PlayerRating,
{
    /// Generate the outcome of a single delivery.
    /// TODO: Incoporate variable/dynamic strategies, field conditions, etc.
    /// TODO: Should return a Result
    fn generate_delivery(&self, rng: &mut impl Rng, state: GameSnapshot<R>) -> DeliveryOutcome;
}
