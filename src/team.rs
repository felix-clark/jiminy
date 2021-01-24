//! Teams of players
use crate::player::Player;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
}

impl Team {
    pub fn batting_order(&self) -> BattingOrder {
        let n_batters = self.players.len();
        let remaining: Vec<usize> = (0..n_batters).rev().collect();
        BattingOrder {
            batters: &self.players,
            remaining,
        }
    }
}

/// Tracks the batting order. This must be able to change mid-game to adjust strategy
/// (only for batters who have not yet batted, of course).
pub struct BattingOrder<'a> {
    /// The reference list of players
    batters: &'a Vec<Player>,
    /// Indices of remaining batters in reverse order. (This allows for convenient
    /// popping.)
    remaining: Vec<usize>,
}

impl<'a> BattingOrder<'a> {
    /// Return a Vec of players remaining in the order that satisfy the given query
    // TODO: Consider returning an impl Iterator instead of collecting into a Vec. This
    // is complicated due to the lifetime constraints.
    pub fn query_remaining(&self, query: &dyn Fn(&Player) -> bool) -> Vec<&'a Player> {
        // TODO: Define whether this should be reversed
        self.remaining
            .iter()
            .filter_map(|&i| {
                let batter: &'a Player = &self.batters[i];
                if query(batter) {
                    Some(batter)
                } else {
                    None
                }
            })
            .collect()
    }

    // TODO: Functions to modify the remaining order
}

impl<'a> Iterator for BattingOrder<'a> {
    type Item = &'a Player;

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.pop().map(|i| &self.batters[i])
    }
}
