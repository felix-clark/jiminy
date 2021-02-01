//! Cricket simulation engine
#[macro_use]
extern crate prettytable;

pub mod error;
pub mod form;
pub mod game;
pub mod model;
pub mod player;
pub mod team;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use crate::team::Team;
    use model::PlayerRatingNull;
    use player::PlayerDb;
    use rand::thread_rng;

    fn test_team(db: &mut PlayerDb<PlayerRatingNull>, id: u16, label: &str) -> Result<Team> {
        const N_PLAYERS: usize = 11;
        let name = format!("team_{}", label);
        let player_names = (0..N_PLAYERS)
            .into_iter()
            .map(|i| format!("{}_{}", label, i));
        let players = player_names
            .map(|n| {
                let player = db.add(n, PlayerRatingNull::default())?;
                Ok((player.id, player.name.clone()))
            })
            .collect::<Result<_>>()?;
        Ok(Team { id, name, players })
    }

    #[test]
    fn sim() -> Result<()> {
        use model::{Model, NullModel};
        let rules = form::Form::test();
        let mut db = PlayerDb::new();
        let team_a = test_team(&mut db, 1, "AUS")?;
        let team_b = test_team(&mut db, 5, "NZ")?;
        let db = db;
        let mut state = game::GameState::new(rules, &team_a, &team_b)?;
        let mut rng = thread_rng();
        let model = NullModel {};

        while !state.complete() {
            let ball = model.generate_delivery(&mut rng, state.snapshot(&db)?);
            state.update(&ball)?;
        }
        state.print_innings_summary()?;
        Ok(())
    }
}
