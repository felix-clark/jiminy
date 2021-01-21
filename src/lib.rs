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

    fn test_rating() -> rating::PlayerRating {
        use rating::*;
        PlayerRating {
            batting: BatRating {},
            bowling: BowlRating {},
            fielding: FieldRating {},
        }
    }

    fn test_team(label: &str) -> team::Team {
        const N_PLAYERS: usize = 11;
        let name = format!("team_{}", label);
        let player_names = (0..N_PLAYERS)
            .into_iter()
            .map(|i| format!("{}_{}", label, i));
        let players = player_names
            .map(|name| team::Player {
                name,
                rating: test_rating(),
            })
            .collect();
        team::Team { name, players }
    }

    #[test]
    fn sim() {
        let rules = form::Form::test();
        let team_a = test_team("A");
        let team_b = test_team("B");
        let mut state = game::GameState::new(rules, team_a, team_b);
        let mut rng = thread_rng();
        while !state.complete() {
            let ball = sim::test_rand_delivery(&mut rng);
            state.update(&ball);
        }
        dbg!(state);
    }
}
