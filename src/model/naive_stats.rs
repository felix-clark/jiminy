//! A model that just uses the batters' and bowlers' averages

use super::{null::FieldRatingNull, Model, PlayerRating};
use crate::game::{DeliveryOutcome, GameSnapshot};
use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
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
    // fours per ball
    pub r4: f32,
    // sixes per ball
    pub r6: f32,
}

impl BatRatingNaiveStats {
    pub fn from_career_stats(
        balls_faced: u32,
        outs: u32,
        runs: u32,
        fours: u32,
        sixes: u32,
    ) -> Self {
        let bf = balls_faced as f32;
        let runs = runs as f32;
        let avg = runs / outs as f32;
        let sr = runs / bf;
        let r4 = fours as f32 / bf;
        let r6 = sixes as f32 / bf;
        Self { avg, sr, r4, r6 }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BowlRatingNaiveStats {
    // Balls per wicket
    pub sr: f32,
    // Runs per wicket
    pub avg: f32,
    // TODO: wide/noball rates
}

impl BowlRatingNaiveStats {
    pub fn from_career_stats(deliveries: u32, wickets: u32, runs_allowed: u32) -> Self {
        let wickets = wickets as f32;
        let sr = deliveries as f32 / wickets;
        let avg = runs_allowed as f32 / wickets;
        Self { sr, avg }
    }
}

pub struct NaiveStatsModel {}

impl Model<PlayerRatingNaiveStats> for NaiveStatsModel {
    fn generate_delivery(
        &self,
        rng: &mut impl Rng,
        state: GameSnapshot<PlayerRatingNaiveStats>,
    ) -> DeliveryOutcome {
        let striker = state.striker;
        let bowler = state.bowler;
        let batter_rating = &striker.rating.batting;
        let bowler_rating = &bowler.rating.bowling;

        let bat_wkt_prob = batter_rating.sr * 0.01 / batter_rating.avg;
        let bowl_wkt_prob = 1. / bowler_rating.sr;
        let wkt_prob = avg_probs(bat_wkt_prob, bowl_wkt_prob);

        // run rates given that no wicket was taken
        let bat_run_rate = batter_rating.sr * 0.01 / (1. - bat_wkt_prob);
        let bowl_run_rate = bowler_rating.avg / (bowler_rating.sr - 1.);
        let run_rate = (bat_run_rate * bowl_run_rate).sqrt();
        // NOTE: boundary fraction is, on average, roughly 50% +/- 5% in all formats.
        // It's strangely not monotonic: lowest in ODI.
        // The fraction of boundaries that are sixes is much higher in more limited overs.
        let four_rate = batter_rating.r4;
        let six_rate = batter_rating.r6;
        // Run rate without boundaries; ignore contribution from bowler
        let run_rate_nb = run_rate - 4. * four_rate - 6. * six_rate;
        assert!(run_rate_nb > 0.);
        assert!(run_rate_nb < 1.);

        // assume for no reason that 10% of non-boundary runs are twos. neglect threes.
        let tr = 0.1;
        let two_rate = 0.5 * run_rate_nb * tr;
        let one_rate = run_rate_nb * (1. - tr);

        // TODO: account for other types of wickets

        let dot_prob = 1.0 - wkt_prob - one_rate - two_rate - four_rate - six_rate;
        assert!(dot_prob > 0.);
        assert!(dot_prob < 1.);

        // mutability for efficient taking of selected element
        let mut outcomes: Vec<(f32, DeliveryOutcome)> = vec![
            (dot_prob, DeliveryOutcome::dot()),
            (one_rate, DeliveryOutcome::running(1)),
            (two_rate, DeliveryOutcome::running(2)),
            (four_rate, DeliveryOutcome::four()),
            (six_rate, DeliveryOutcome::six()),
            (
                0.5 * wkt_prob,
                DeliveryOutcome::bowled(striker.id, &bowler.name),
            ),
            (
                0.5 * wkt_prob,
                DeliveryOutcome::lbw(striker.id, &bowler.name),
            ),
        ];
        let d = WeightedIndex::new(outcomes.iter().map(|i| i.0)).unwrap();
        let choice = d.sample(rng);
        let outcome = outcomes.swap_remove(choice).1;
        outcome
    }
}

/// Return the average of two probabilities (on a logistic scale)
fn avg_probs(p1: f32, p2: f32) -> f32 {
    // TODO: fix this for p_[12] = 1 (two sqrt terms)
    let avg_odds = f32::sqrt(p1 * p2 / ((1. - p1) * (1. - p2)));
    avg_odds / (1. + avg_odds)
}
