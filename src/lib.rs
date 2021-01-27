//! Cricket simulation engine
#[macro_use]
extern crate prettytable;

pub mod form;
pub mod game;
pub mod player;
pub mod rating;
pub mod sim;
pub mod team;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    fn test_team(id: u16, label: &str) -> team::Team {
        const N_PLAYERS: usize = 11;
        let name = format!("team_{}", label);
        let player_names = (0..N_PLAYERS)
            .into_iter()
            .map(|i| format!("{}_{}", label, i));
        let players = player_names.map(player::Player::new).collect();
        team::Team { id, name, players }
    }

    #[test]
    fn sim() {
        use sim::{Model, TestModel};
        let rules = form::Form::test();
        let team_a = test_team(1, "AUS");
        let team_b = test_team(5, "NZ");
        let mut state = game::GameState::new(rules, &team_a, &team_b);
        let mut rng = thread_rng();
        let model = TestModel {};

        while !state.complete() {
            let ball = model.generate_delivery(&mut rng, &state);
            state.update(&ball);
        }
        state.print_innings_summary();
    }
}
