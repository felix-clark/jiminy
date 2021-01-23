//! Simulation

use crate::game::{DeliveryOutcome, Dismissal, Runs};
use rand::{distributions::Uniform, Rng};

pub fn test_rand_delivery(rng: &mut impl Rng) -> DeliveryOutcome {
    let dist = Uniform::new(0., 1.);
    let rand: f64 = rng.sample(dist);
    if rand < 0.02 {
        DeliveryOutcome {
            wicket: Some(Dismissal::Caught),
            ..Default::default()
        }
    } else if rand <= 0.4 {
        DeliveryOutcome {
            runs: Runs::Running(1),
            ..Default::default()
        }
    } else {
        DeliveryOutcome::default()
    }
}
