//! A first attempt at a non-trivial model
use serde::{Deserialize, Serialize};

// NOTE:
// Inspiration could be taken from baseball's scout ratings system where league average rating of a
// tool is 50 and the standard deviation is 10. However, since athletes are selected from the
// extreme end of the talent distribution, there should be many more below-average athletes than
// above-average athletes available (including bubble and sub-pro) so the distribution of
// tool/overall scores can probably be modeled as exponential, with a level-dependent cutoff. 
// The gamma distribution may be a useful tool here, as it generalizes the exponential distribution
// for shape parameter != 1, effectively describing a soft lower bound.

/// Ratings for batting
#[derive(Debug, Deserialize, Serialize)]
pub struct BatRatingAlpha {
    // avoid wickets (eye + contact?)
    defense: u8,
    // may be redundant concept
    // TODO: consider this as a status, or ball-by-ball stat
    // eye: u8,
    // ability to hit ball for runs
    contact: u8,
    // control or ability to put it in gap, largely to determine 4s
    gap: u8,
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
