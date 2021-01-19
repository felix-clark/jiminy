//! Cricket simulation engine

pub mod form;
pub mod game;
pub mod rating;
pub mod sim;
pub mod team;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    #[test]
    fn sim() {
        let rules = form::Form::test();
        let mut state = game::GameState::new(rules);
        let mut rng = thread_rng();
        while !state.complete() {
            let ball = sim::test_rand_delivery(&mut rng);
            state.update(&ball);
        }
        dbg!(state);
    }
}
