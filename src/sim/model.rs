//! The interface and implementations for the cricket model(s)
use crate::game::{DeliveryOutcome, GameState};
use rand::Rng;

pub trait Model {
    /// Generate the outcome of a single delivery.
    /// TODO: Incoporate variable/dynamic strategies, field conditions, etc.
    /// TODO: Should return a Result
    fn generate_delivery(&self, rng: &mut impl Rng, state: &GameState) -> DeliveryOutcome;
}
