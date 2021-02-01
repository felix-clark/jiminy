//! A model that just uses the batters' and bowlers' averages

use super::null::FieldRatingNull;
use super::{Model, PlayerRating};
use crate::game::{DeliveryOutcome, Dismissal, GameSnapshot, Runs};
use rand::{distributions as dist, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerRatingNaiveStats {
    pub batting: BatRatingNaiveStats,
    pub bowling: BowlRatingNaiveStats,
    // No stats for fielding right now
    pub fielding: FieldRatingNull,
}
impl PlayerRating for PlayerRatingNaiveStats {}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatRatingNaiveStats {
    // Runs per wicket
    pub avg: f32,
    // Runs per 100 balls
    pub sr: f32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct BowlRatingNaiveStats {
    // Balls per wicket
    pub sr: f32,
    // Runs per wicket
    pub avg: f32,
}

pub struct NaiveStatsModel {}

impl Model<PlayerRatingNaiveStats> for NaiveStatsModel {
    fn generate_delivery(
        &self,
        rng: &mut impl Rng,
        state: GameSnapshot<PlayerRatingNaiveStats>,
    ) -> DeliveryOutcome {
        let batter_rating = &state.striker.rating.batting;
        let bowler_rating = &state.bowler.rating.bowling;
        let bat_wkt_prob = batter_rating.sr * 0.01 / batter_rating.avg;
        let bowl_wkt_prob = 1. / bowler_rating.sr;
        let wkt_prob = avg_probs(bat_wkt_prob, bowl_wkt_prob);
        todo!()
    }
}

/// Return the average of two probabilities (on a logistic scale)
fn avg_probs(p1: f32, p2: f32) -> f32 {
    // TODO: fix this for p_[12] = 1 (two sqrt terms)
    let avg_odds = f32::sqrt(p1 * p2 / ((1. - p1) * (1. - p2)));
    avg_odds / (1. + avg_odds)
}
