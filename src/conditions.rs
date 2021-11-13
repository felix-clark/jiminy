//! Conditions of a match such as weather and ball state

/// The style and manufacturer of the cricket ball
#[derive(Debug, Clone, Copy)]
pub enum BallType {
    /// Used in test matches.
    /// TODO: Split into manufacturer, i.e.
    ///   Kookabura for test matches in Aus/NZ/Zimb/SriLanka/SA/Pak
    ///   Dukes for England/WI
    ///   SG for India
    RedLeather,
    /// White ball is used in limited overs for visibility under floodlights.
    WhiteLeather,
    // Pink leather is used for day-night tests
    // PinkLeather, ...
    // Could add non-professional types like cork, synthetic, tennis, etc.
}

/// Style and conditions of a ball
#[derive(Debug, Clone)]
pub struct Ball {
    /// The style of ball
    pub ball_type: BallType,
    /// A proxy for wear-and-tear due to scuffing the pitch
    pub deliveries: u16,
    /// A proxy for wear-and-tear due to batting
    pub runs: u16,
}

#[derive(Debug, Clone)]
pub struct Weather {}

/// Tracks other conditions not related to the players or sides
#[derive(Debug, Clone)]
pub struct Conditions {
    pub ball: Ball,
    pub weather: Weather,
    // TODO: Pitch characteristics
}
