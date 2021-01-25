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

    fn test_team(label: &str) -> team::Team {
        const N_PLAYERS: usize = 11;
        let name = format!("team_{}", label);
        let player_names = (0..N_PLAYERS)
            .into_iter()
            .map(|i| format!("{}_{}", label, i));
        let players = player_names.map(player::Player::new).collect();
        team::Team { name, players }
    }

    #[test]
    fn sim() {
        let rules = form::Form::test();
        let team_a = test_team("AUS");
        let team_b = test_team("NZ");
        let mut state = game::GameState::new(rules, &team_a, &team_b);
        let mut rng = thread_rng();

        while !state.complete() {
            let ball = sim::test_rand_delivery(&mut rng);
            state.update(&ball);
        }
        state.print_innings_summary();
    }
}
