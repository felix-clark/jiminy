//! Player data and identification

use crate::rating::PlayerRating;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

type Id = usize;
static PLAYER_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Retrieve a new unique player ID
fn get_new_player_id() -> Id {
    // NOTE: This choice of ordering hasn't been considered.
    PLAYER_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Deserialize, Serialize)]
// #[serde(deny_unknown_fields)] // This makes an error if additional fields are present
pub struct Player {
    // TODO: consider using team + cap number to identify test players, although this
    // will not cover cricketers who have not made a test appearance.
    #[serde(skip, default = "get_new_player_id")]
    id: Id,
    pub name: String,
    pub rating: PlayerRating,
}

impl Player {
    // TODO: Should take a rating as well
    pub fn new(name: String) -> Self {
        use crate::rating::*;

        let id = get_new_player_id();
        Self {
            id,
            name,
            rating: PlayerRating {
                batting: BatRating {},
                bowling: BowlRating {},
                fielding: FieldRating {},
            },
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Player {}
