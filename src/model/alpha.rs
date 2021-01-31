//! A first attempt at a non-trivial model
use serde::{Deserialize, Serialize};

/// Ratings for batting
#[derive(Debug, Deserialize, Serialize)]
pub struct BatRatingAlpha {
    // avoid wickets (eye + contact?)
    //defense: u8,
    // may be redundant concept
    // TODO: consider this as a status, or ball-by-ball stat
    eye: u8,
    contact: u8,
    // ability to put it in gap
    control: u8,
    // sixes, possibly 4s
    power: u8,
}
impl Default for BatRatingAlpha {
    fn default() -> Self {
        Self {
            eye: 0,
            contact: 0,
            control: 0,
            power: 0,
        }
    }
}

/// Ratings for bowling
#[derive(Debug, Deserialize, Serialize)]
pub struct BowlRatingAlpha {
    // take bowling wickets. Perhaps should be composite of others.
    //attack: u8,
    // affects reaction time and eye (pace/fast bowling)
    velocity: u8,
    // ability to place ball (redundant w/ attack?)
    // TODO: Should this be split into vertical and horizontal control?
    control: u8,
    // While velo + control (length control) should affect the batter's ability to get set on
    // front/back, swing and spin should affect their sideways eye.
    // movement in the air (fast, medium-fast)
    swing: u8,
    // movement off the ground
    spin: u8,
}
impl Default for BowlRatingAlpha {
    fn default() -> Self {
        Self {
            velocity: 0,
            control: 0,
            swing: 0,
            spin: 0,
        }
    }
}
