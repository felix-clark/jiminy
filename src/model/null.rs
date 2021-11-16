//! A model that doesn't depend on any data
use super::{Model, PlayerRating};
use crate::game::{DeliveryOutcome, GameSnapshot};
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerRatingNull {
    pub batting: BatRatingNull,
    pub bowling: BowlRatingNull,
    pub fielding: FieldRatingNull,
}
impl Default for PlayerRatingNull {
    fn default() -> Self {
        Self {
            batting: BatRatingNull {},
            bowling: BowlRatingNull {},
            fielding: FieldRatingNull {},
        }
    }
}
impl PlayerRating for PlayerRatingNull {}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatRatingNull {}
#[derive(Debug, Deserialize, Serialize)]
pub struct BowlRatingNull {}
#[derive(Debug, Deserialize, Serialize)]
pub struct FieldRatingNull {}

/// A very simple model that doesn't use player stats
pub struct NullModel {}

impl Model<PlayerRatingNull> for NullModel {
    fn generate_delivery(
        &self,
        rng: &mut impl Rng,
        state: GameSnapshot<PlayerRatingNull>,
    ) -> DeliveryOutcome {
        let striker_id = state.striker.id;
        let bowler = state.bowler;
        // NOTE: Consider WeightedIndex distribution instead of manually cutting on a standard
        // uniform value
        let dist = Uniform::new(0., 1.);
        let rand: f64 = rng.sample(dist);
        if rand < 0.01 {
            DeliveryOutcome::caught(striker_id, &bowler.name, "?fielder")
        } else if rand <= 0.015 {
            DeliveryOutcome::bowled(striker_id, &bowler.name)
        } else if rand <= 0.02 {
            DeliveryOutcome::lbw(striker_id, &bowler.name)
        } else if rand <= 0.4 {
            DeliveryOutcome::running(1)
        } else if rand <= 0.42 {
            DeliveryOutcome::four()
        } else if rand <= 0.424 {
            DeliveryOutcome::six()
        } else {
            DeliveryOutcome::dot()
        }
    }
}
