//! Struct to define the format of a match

use crate::conditions::{Ball, BallType};

/// Defines the format of a match
#[derive(Debug)]
pub struct Form {
    /// The type and style of ball used.
    pub ball_type: BallType,
    /// The number of turns each side has to bat.
    pub innings: u8,
    /// The number of overs in an innings, if limited
    pub overs_per_innings: Option<u16>,
    /// The number of balls per over.
    pub balls_per_over: u8,
    // TODO: time limit, days/hours?
    // TODO: ball type/color
    // TODO: overs for new ball (80 in test)?
    // TODO: fielding restrictions/powerplays
    // TODO: players per side (almost always 11)?
    pub batsmen_per_side: u8,
    // TODO: Maximum overs per bowler (10 in ODI?)
}

impl Default for Form {
    fn default() -> Self {
        Self {
            innings: 2,
            overs_per_innings: None,
            balls_per_over: 6,
            batsmen_per_side: 11,
            ball_type: BallType::RedLeather,
        }
    }
}

impl Form {
    /// Standard test format. Could also be called first_class.
    pub fn test() -> Self {
        Self::default()
    }

    /// List-A, e.g. One Day International (ODI)
    pub fn odi() -> Self {
        Self {
            innings: 1,
            overs_per_innings: Some(50),
            ball_type: BallType::WhiteLeather,
            ..Default::default()
        }
    }

    /// Twenty20
    pub fn t20() -> Self {
        Self {
            innings: 1,
            overs_per_innings: Some(20),
            ball_type: BallType::WhiteLeather,
            ..Default::default()
        }
    }

    /// Generate a fresh ball
    pub(crate) fn new_ball(&self) -> Ball {
        Ball {
            ball_type: self.ball_type,
            deliveries: 0,
            runs: 0,
        }
    }
}
