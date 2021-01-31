//! A model that doesn't depend on any data
use super::{Model, PlayerRating};
use crate::game::{DeliveryOutcome, Dismissal, GameSnapshot, Runs};
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

impl NullModel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Model<PlayerRatingNull> for NullModel {
    fn generate_delivery(
        &self,
        rng: &mut impl Rng,
        state: GameSnapshot<PlayerRatingNull>,
    ) -> DeliveryOutcome {
        let bowler = state.bowler;
        let dist = Uniform::new(0., 1.);
        let rand: f64 = rng.sample(dist);
        if rand < 0.01 {
            DeliveryOutcome {
                wicket: Some(Dismissal::Caught(
                    "?fielder".to_string(),
                    "?bowler".to_string(),
                )),
                ..Default::default()
            }
        } else if rand <= 0.015 {
            DeliveryOutcome {
                wicket: Some(Dismissal::Bowled(bowler.name.to_string())),
                ..Default::default()
            }
        } else if rand <= 0.02 {
            DeliveryOutcome {
                wicket: Some(Dismissal::Lbw(bowler.name.to_string())),
                ..Default::default()
            }
        } else if rand <= 0.4 {
            DeliveryOutcome {
                runs: Runs::Running(1),
                ..Default::default()
            }
        } else if rand <= 0.42 {
            DeliveryOutcome {
                runs: Runs::Four,
                ..Default::default()
            }
        } else if rand <= 0.424 {
            DeliveryOutcome {
                runs: Runs::Six,
                ..Default::default()
            }
        } else {
            DeliveryOutcome::default()
        }
    }
}
