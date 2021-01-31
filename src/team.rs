//! Teams of players
use crate::{
    model::PlayerRating,
    player::{Player, PlayerDb, PlayerId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Team {
    pub id: u16,
    pub name: String,
    /// The UIDs and names of the players
    pub players: Vec<(PlayerId, String)>,
}

impl Team {
    pub fn batting_order(&self) -> BattingOrder {
        let n_batters = self.players.len();
        let remaining: Vec<usize> = (0..n_batters).rev().collect();
        let batters = self.players.iter().map(|(id, _)| id).cloned().collect();
        BattingOrder { batters, remaining }
    }

    pub fn bowlers(&self) -> Bowlers {
        let bowlers: Vec<PlayerId> = self.players[5..11]
            .iter()
            .map(|(id, _)| id)
            .rev()
            .cloned()
            .collect();
        let last: PlayerId = bowlers[1];
        Bowlers { bowlers, last }
    }

    pub fn get_name(&self, id: PlayerId) -> Option<&str> {
        self.players
            .iter()
            .find(|(i, _)| i == &id)
            .map(|(_, n)| n.as_str())
    }
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Team {}

/// Tracks the batting order. This must be able to change mid-game to adjust strategy
/// (only for batters who have not yet batted, of course).
pub struct BattingOrder {
    /// The reference list of players
    batters: Vec<PlayerId>,
    /// Indices of remaining batters in reverse order. (This allows for convenient
    /// popping.)
    remaining: Vec<usize>,
}

impl BattingOrder {
    /// Return a Vec of players remaining in the order that satisfy the given query
    // TODO: Consider returning an impl Iterator instead of collecting into a Vec. This
    // is complicated due to the lifetime constraints.
    pub fn query_remaining<R>(
        &self,
        db: PlayerDb<R>,
        query: &dyn Fn(&Player<R>) -> bool,
    ) -> Vec<PlayerId>
    where
        R: PlayerRating,
    {
        // TODO: Define whether this should be reversed
        self.remaining
            .iter()
            .filter_map(|&i| {
                let batter: PlayerId = self.batters[i];
                if query(db.get(batter).expect("Not in db")) {
                    Some(batter)
                } else {
                    None
                }
            })
            .collect()
    }

    // TODO: Functions to modify the remaining order
}

impl Iterator for BattingOrder {
    type Item = PlayerId;

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.pop().map(|i| self.batters[i])
    }
}

/// Iterates through available bowlers
// TODO: Incorporate various strategies
pub struct Bowlers {
    pub bowlers: Vec<PlayerId>,
    /// The previous bowler so that we don't repeat
    last: PlayerId,
}

impl Bowlers {
    // TODO: methods to adjust strategy (?)
}

impl Iterator for Bowlers {
    type Item = PlayerId;

    fn next(&mut self) -> Option<Self::Item> {
        // Right now just switch between the top two bowlers
        let bowler: PlayerId = self
            .bowlers
            .iter()
            .find(|&&b| self.last != b)
            .cloned()
            .unwrap();
        self.last = bowler;
        Some(bowler)
    }
}
