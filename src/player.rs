//! Player data and identification

use crate::rating::PlayerRating;
use std::sync::atomic::{AtomicUsize, Ordering};

type Id = usize;
static PLAYER_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Retrieve a new unique player ID
fn get_new_player_id() -> usize {
    // NOTE: This choice of ordering hasn't been considered.
    PLAYER_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct Player {
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
