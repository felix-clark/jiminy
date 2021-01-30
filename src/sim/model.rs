//! The interface and implementations for the cricket model(s)
use crate::{
    game::{DeliveryOutcome, GameState},
    player::PlayerDb,
    rating::PlayerRating,
};
use rand::Rng;

pub trait Model<R>
where
    R: PlayerRating,
{
    /// Generate the outcome of a single delivery.
    /// TODO: Incoporate variable/dynamic strategies, field conditions, etc.
    /// TODO: Should return a Result
    fn generate_delivery(
        &self,
        rng: &mut impl Rng,
        db: &PlayerDb<R>,
        state: &GameState,
    ) -> DeliveryOutcome;
}
