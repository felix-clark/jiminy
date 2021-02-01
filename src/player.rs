//! Player data and identification

use crate::{
    error::{Error, Result},
    model::PlayerRating,
};
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

pub type PlayerId = usize;
static PLAYER_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Retrieve a new unique player ID
fn get_new_player_id() -> PlayerId {
    // NOTE: This choice of ordering hasn't been considered.
    PLAYER_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub struct PlayerDb<R>
where
    R: PlayerRating,
{
    map: FnvHashMap<PlayerId, Player<R>>,
}

impl<R> PlayerDb<R>
where
    R: PlayerRating,
{
    pub fn new() -> Self {
        Self {
            map: FnvHashMap::default(),
        }
    }
    pub fn get(&self, id: PlayerId) -> Option<&Player<R>> {
        self.map.get(&id)
    }

    pub fn add(&mut self, name: String, rating: R) -> Result<&Player<R>> {
        let id = get_new_player_id();
        let player = Player { id, name, rating };
        if let Some(p) = self.map.insert(player.id, player) {
            return Err(Error::DuplicatePlayerId(p.id));
        }
        Ok(self.map.get(&id).unwrap())
    }
}

#[derive(Debug, Deserialize, Serialize)]
// #[serde(deny_unknown_fields)] // This makes an error if additional fields are present
pub struct Player<R>
where
    R: PlayerRating,
{
    // TODO: consider using team + cap number to identify test players, although this
    // will not cover cricketers who have not made a test appearance.
    #[serde(skip, default = "get_new_player_id")]
    pub id: PlayerId,
    pub name: String,
    pub rating: R,
}

impl<R> PartialEq for Player<R>
where
    R: PlayerRating,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<R> Eq for Player<R> where R: PlayerRating {}
