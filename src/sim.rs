//! Simulation and models
pub mod model;
pub use model::Model;

use crate::{
    game::{DeliveryOutcome, Dismissal, Runs},
    player::Player,
};
use rand::{distributions::Uniform, Rng};

/// A very simple model that doesn't use player stats
pub struct TestModel {}

impl Model for TestModel {
    fn generate_delivery(
        &self,
        rng: &mut impl Rng,
        state: &crate::game::GameState,
    ) -> DeliveryOutcome {
        let bowler: &Player = state.bowler().expect("Could not get bowler");
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
