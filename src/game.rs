//! Description of the state and events of a match.
use crate::form;

// For now just use a boolean as a flag for the team
type TeamId = bool;

// TODO: Format the output instead of deriving Debug
#[derive(Debug)]
pub struct GameState {
    /// The rules of the match
    form: form::Form,
    /// The number of innings completed
    innings: u8,
    /// TODO: This will need to be more sophisticated to account for which batters are out
    wickets: u8,
    /// The number of overs that have been completed
    overs: u16,
    /// The number of legal balls delivered in the over
    balls: u8,
    /// Which team is up to bat
    batting_team: TeamId,
    /// runs score by the batting team this innings
    innings_runs: u16,
    /// Scores completed in previous innings for each team
    /// TODO: count wickets as well (probably use a new struct)
    previous_innings_runs_a: Vec<u16>,
    previous_innings_runs_b: Vec<u16>,
}

impl GameState {
    pub fn new(rules: form::Form) -> Self {
        Self {
            form: rules,
            innings: 0,
            wickets: 0,
            overs: 0,
            balls: 0,
            batting_team: false,
            innings_runs: 0,
            previous_innings_runs_a: Vec::new(),
            previous_innings_runs_b: Vec::new(),
        }
    }

    /// Whether the match is finished
    pub fn complete(&self) -> bool {
        // TODO: Match can also be complete when one team is leading and the other team
        // has no more chances to bat.
        // TODO: It can also finish due to time, which is currently not tracked.
        self.innings >= 2 * self.form.innings
    }

    /// Batting team declares to complete their innings
    pub fn declare(&mut self) {
        todo!("Reset balls/overs/innings counters");
    }

    /// Update the game state based on the outcome of a delivery
    pub fn update(&mut self, ball: &DeliveryOutcome) {
        let runs = ball.runs.runs() as u16;
        // TODO: Some of these should count against the bowler, and others not.
        // Account for this when tracking individual stats.
        let extras = ball.extras.iter().map(Extra::runs).sum::<u8>() as u16;
        self.innings_runs += runs + extras;

        // TODO: account for which batsmen is out
        if ball.wicket.is_some() {
            self.wickets += 1;
        }

        if ball.legal() {
            self.balls += 1;
        }
        if self.balls >= self.form.balls_per_over {
            self.balls = 0;
            self.overs += 1;
        }

        // Check if we need to change to a new innings
        let mut new_innings = false;
        if self.wickets + 1 >= self.form.batsmen_per_side {
            new_innings = true;
        }
        if let Some(opi) = self.form.overs_per_innings {
            if self.overs >= opi {
                new_innings = true;
            }
        }
        if new_innings {
            self.new_innings();
        }
    }

    /// Initiate a new innings
    fn new_innings(&mut self) {
        self.balls = 0;
        self.overs = 0;
        self.wickets = 0;
        self.innings += 1;
        if self.batting_team {
            self.previous_innings_runs_a.push(self.innings_runs);
        } else {
            self.previous_innings_runs_b.push(self.innings_runs);
        }
        self.innings_runs = 0;
        // TODO: The batting team doesn't always switch if the trailing team is made to
        // go again (down by ~150+ runs)
        self.batting_team = !self.batting_team;
    }
}

/// Methods of dismissal
pub enum Dismissal {
    /// Legitimate delivery hits wicket and puts it down.
    Bowled,
    /// Ball is hit in the air and caught in-bounds
    Caught,
    /// A delivery that would have hit the wickets instead first makes contact with the
    /// striker (not the bat).
    LegBeforeWicket,
    /// The striker is put out while running
    RunOutStriker,
    /// The only method by which the non-striker can be dismissed.
    RunOutNonStriker,
    /// The wicket-keeper puts down the wicket while the striker is out of the crease.
    /// Takes precedence over run-out.
    Stumped,
    // TODO: rare dismissals
}

/// Normal runs
pub enum Runs {
    /// Runs acquired by running. Batsmen change ends if this is odd.
    /// This includes dots (value of 0)
    Running(u8),
    /// Ball reaches boundary after bouncing
    Four,
    /// Ball crosses boundary in the air
    Six,
}

impl Runs {
    pub fn runs(&self) -> u8 {
        use Runs::*;
        match &self {
            Running(n) => *n,
            Four => 4,
            Six => 6,
        }
    }
}

/// Extra runs
pub enum Extra {
    /// One penalty run. Additional runs can still be scored off a no-ball.
    NoBall,
    /// One penalty run is awarded if the ball is judged to not be hittable with a
    /// normal cricket swing. A wide that is also a no-ball is counted as a no-ball.
    Wide,
    /// A bye in which the batsman does not make contact and the wicket is not made. Runs
    /// can be scored but this is often zero. Balls that make it to the boundary are
    /// scored as fours. Byes and leg byes can be scored from no-balls and wides.
    Bye(Runs),
    /// Similar to a bye, but with contact off the batter (not the bat) that is not LBW.
    LegBye(Runs),
    /// Penalty runs can also be awarded for various breaches of conduct.
    Penalty(u8),
}

impl Extra {
    pub fn runs(&self) -> u8 {
        use Extra::*;
        match &self {
            NoBall | Wide => 1,
            Bye(runs) | LegBye(runs) => runs.runs(),
            Penalty(n) => *n,
        }
    }
}

/// The outcome of a single delivery. Also known as a "ball", although a delivery can
/// result in a no-ball.
pub struct DeliveryOutcome {
    /// Whether a batsman is dismissed along with the method. In standard cricket the
    /// ball is dead upon a dismissal so there are no double-plays.
    pub wicket: Option<Dismissal>,
    /// Runs scored by batting the ball into play
    pub runs: Runs,
    /// Any extra runs accrued on the play
    pub extras: Vec<Extra>,
}

impl DeliveryOutcome {
    /// Whether the delivery should count as a legal ball
    pub fn legal(&self) -> bool {
        use Extra::*;
        !self.extras.iter().any(|ex| matches!(ex, NoBall | Wide))
    }
}

impl Default for DeliveryOutcome {
    fn default() -> Self {
        Self {
            wicket: None,
            runs: Runs::Running(0),
            extras: Vec::new(),
        }
    }
}