//! Description of the state and events of a match.
use crate::team::{BattingOrder, Team};
use crate::{form, player::Player};

use std::{fmt::Display, mem};

pub struct GameState<'a> {
    /// The rules of the match
    form: form::Form,
    /// The home team
    team_a: &'a Team,
    /// The visiting team
    team_b: &'a Team,
    /// The number of overs that have been completed
    overs: u16,
    /// The number of legal balls delivered in the over
    balls: u8,
    /// Which team is up to bat
    batting_team: *const Team,
    /// Stats of the currently batting team
    batting_innings_stats: TeamBattingInningsStats<'a>,
    /// Previous innings scores
    // TODO: Consider initializing all of these in advance and using an index/pointer to the current one
    // TODO: Or, store team in the stats struct and collect into a single Vec
    previous_innings_a: Vec<TeamBattingInningsStats<'a>>,
    previous_innings_b: Vec<TeamBattingInningsStats<'a>>,
}

impl<'a> GameState<'a> {
    pub fn new(rules: form::Form, team_a: &'a Team, team_b: &'a Team) -> Self {
        let batting_innings_stats = TeamBattingInningsStats::new(team_a);
        Self {
            form: rules,
            team_a,
            team_b,
            overs: 0,
            balls: 0,
            batting_team: team_a,
            batting_innings_stats,
            previous_innings_a: Vec::new(),
            previous_innings_b: Vec::new(),
        }
    }

    /// Whether the match is finished
    pub fn complete(&self) -> bool {
        // TODO: Match can also be complete when one team is leading and the other team
        // has no more chances to bat.
        // TODO: It can also finish due to time, which is currently not tracked.
        self.previous_innings_a.len() as u8 >= self.form.innings
            && self.previous_innings_b.len() as u8 >= self.form.innings
    }

    /// Batting team declares to complete their innings
    pub fn declare(&mut self) {
        todo!("Reset balls/overs/innings counters");
    }

    /// Update the game state based on the outcome of a delivery
    pub fn update(&mut self, ball: &DeliveryOutcome) {
        self.batting_innings_stats.update(&ball);

        // TODO: Count these in TeamBattingInningsStats
        if ball.legal() {
            self.balls += 1;
        }
        if self.balls >= self.form.balls_per_over {
            self.balls = 0;
            self.overs += 1;
            self.batting_innings_stats.switch_striker();
        }

        // Check if we need to change to a new innings
        let mut new_innings = false;
        // if self.wickets + 1 >= self.form.batsmen_per_side {
        // TODO: Need to pass batsmen_per_side to team lineup
        if self.batting_innings_stats.all_out() {
            assert_eq!(
                self.batting_innings_stats.wickets() + 1,
                self.form.batsmen_per_side
            );
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
        // TODO: The batting team doesn't always switch if the trailing team is made to
        // go again (down by ~150+ runs)
        let next_batting_team: &Team = if self.batting_team == self.team_a {
            &self.team_b
        } else if self.batting_team == self.team_b {
            &self.team_a
        } else {
            panic!("Should be one of these two teams")
        };
        if self.batting_team == self.team_a {
            self.previous_innings_a.push(mem::replace(
                &mut self.batting_innings_stats,
                TeamBattingInningsStats::new(next_batting_team),
            ));
        } else {
            self.previous_innings_b.push(mem::replace(
                &mut self.batting_innings_stats,
                TeamBattingInningsStats::new(&*next_batting_team),
            ));
        }
        self.batting_team = next_batting_team;
    }

    /// Print a summary of each innings to stdout
    pub fn print_innings_summary(&self) {
        println!("\n{}:", self.team_a.name);
        for innings in self.previous_innings_a.iter() {
            innings.print_summary();
        }
        println!("\n{}:", self.team_b.name);
        for innings in self.previous_innings_b.iter() {
            innings.print_summary();
        }
    }
}

/// Methods of dismissal
/// TODO: Include information about each dismissal like bowler, which fielder
/// caught/stumped, etc.
#[derive(Clone)]
pub enum Dismissal {
    /// Legitimate delivery hits wicket and puts it down.
    Bowled(String),
    /// Ball is hit in the air and caught in-bounds
    Caught(String, String),
    /// Leg before wicket: A delivery that would have hit the wickets instead first
    /// makes contact with the striker (not the bat).
    Lbw(String),
    /// The striker is put out while running
    // TODO: Consider not distinguishing these, but letting the simulation access both
    RunOutStriker(String),
    /// The only method by which the non-striker can be dismissed.
    RunOutNonStriker(String),
    /// The wicket-keeper puts down the wicket while the striker is out of the crease.
    /// Takes precedence over run-out.
    Stumped(String),
    // TODO: rare dismissals
}

impl Display for Dismissal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Dismissal::*;
        match &self {
            Bowled(bowler) => write!(f, "b {}", bowler),
            Caught(caught, bowler) => write!(f, "c {} b {}", caught, bowler),
            Lbw(bowler) => write!(f, "lbw b {}", bowler),
            RunOutStriker(fielder) | RunOutNonStriker(fielder) => write!(f, "runout ({})", fielder),
            Stumped(keeper) => write!(f, "st {}", keeper),
        }
    }
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

/// The stats of a batter for a single innings
struct BatterInningsStats {
    /// Runs scored by this batter
    pub runs: u16,
    // Extras scored by the team while this batter is up
    // Right now this is only counted at the team-level, which is sufficient for score-keeping.
    // pub extras: u16,
    /// Legal deliveries made to this batter
    pub balls: u16,
    /// Whether the batter had been made out
    pub out: Option<Dismissal>,
    /// Number of fours scored (the runs are also included in self.runs)
    pub fours: u8,
    /// Number of sixes scored (the runs are also included in self.runs)
    pub sixes: u8,
}

impl BatterInningsStats {
    pub fn new() -> Self {
        Self {
            runs: 0,
            balls: 0,
            out: None,
            fours: 0,
            sixes: 0,
        }
    }
}

impl Display for BatterInningsStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.balls == 0 {
            write!(f, "-")
        } else if self.out.is_some() {
            write!(f, "{} ({})", self.runs, self.balls)
        } else {
            write!(f, "{}* ({})", self.runs, self.balls)
        }
    }
}

struct TeamBattingInningsStats<'a> {
    /// Reference to the team's lineup
    pub batting_order: BattingOrder<'a>,
    // pub team: &'a Team,
    /// Individual batting stats
    pub batters: Vec<(&'a Player, BatterInningsStats)>,
    /// Extra runs awarded to the team this inning
    pub extras: u16,
    /// Index of one of the current batters in self.batters
    batter_a: usize,
    /// The other of the current batters
    batter_b: usize,
    // TODO: count balls and overs here as well? (requires reference to rules)
    /// Whether batter_a is the striker
    striker_a: bool,
}

impl<'a> TeamBattingInningsStats<'a> {
    /// Create a new team stats object for a fresh innings
    pub fn new(team: &'a Team) -> Self {
        let mut batting_order = team.batting_order();
        let mut batters = Vec::new();
        batters.push((
            batting_order.next().expect("Not enough batters in order"),
            BatterInningsStats::new(),
        ));
        batters.push((
            batting_order.next().expect("Not enough batters in order"),
            BatterInningsStats::new(),
        ));
        Self {
            batting_order,
            batters,
            extras: 0,
            batter_a: 0,
            batter_b: 1,
            striker_a: true,
        }
    }

    /// Returns true iff the innings is over
    pub fn all_out(&self) -> bool {
        let num_batters = self.batters.len();
        self.batter_a >= num_batters || self.batter_b >= num_batters
    }

    /// Return the total number of team runs
    pub fn team_runs(&self) -> u16 {
        let batter_runs = self.batters.iter().map(|(_, st)| st.runs).sum::<u16>();
        batter_runs + self.extras
    }

    /// Return the total number of wickets
    pub fn wickets(&self) -> u8 {
        self.batters
            .iter()
            .filter(|(_, st)| st.out.is_some())
            .count() as u8
    }

    /// Switch which batter is the striker. This must be done on a new over, and is done
    /// automatically when an odd number of runs are scored.
    pub fn switch_striker(&mut self) {
        self.striker_a = !self.striker_a;
    }

    /// Returns a reference to the current striker
    // pub fn striker(&self) -> Option<&Player> {
    pub fn striker(&self) -> &Player {
        let striker_idx = if self.striker_a {
            self.batter_a
        } else {
            self.batter_b
        };
        assert!(
            striker_idx < self.batters.len(),
            "Innings is over, can't get striker"
        );
        &self.batters[striker_idx].0
    }

    /// Returns a reference to the current non-striker
    pub fn non_striker(&self) -> &Player {
        let non_striker_idx = if self.striker_a {
            self.batter_b
        } else {
            self.batter_a
        };
        assert!(
            non_striker_idx < self.batters.len(),
            "Innings is over, can't get striker"
        );
        &self.batters[non_striker_idx].0
    }

    /// Update the stats of a batter based on a delivery outcome
    pub fn update(&mut self, ball: &DeliveryOutcome) {
        let (striker_idx, non_striker_idx) = if self.striker_a {
            (self.batter_a, self.batter_b)
        } else {
            (self.batter_b, self.batter_a)
        };

        let striker_stats: &mut BatterInningsStats = &mut self.batters[striker_idx].1;
        if ball.legal() {
            striker_stats.balls += 1;
        }

        let mut switch_striker: bool = false;

        // Add runs and extras to the totals
        match ball.runs {
            Runs::Running(x) => {
                if x % 2 == 1 {
                    switch_striker = !switch_striker;
                }
                striker_stats.runs += x as u16;
            }
            Runs::Four => {
                striker_stats.runs += 4;
                striker_stats.fours += 1;
            }
            Runs::Six => {
                striker_stats.runs += 6;
                striker_stats.sixes += 1;
            }
        }
        self.extras += ball.extras.iter().map(|x| x.runs() as u16).sum::<u16>();

        // Switch if bye/leg byes result in an odd number of runs
        for extra in ball
            .extras
            .iter()
            .filter(|ex| matches!(ex, Extra::Bye(_) | Extra::LegBye(_)))
        {
            match extra {
                Extra::Bye(Runs::Running(b)) | Extra::LegBye(Runs::Running(b)) => {
                    if b % 2 == 1 {
                        switch_striker = !switch_striker;
                    }
                }
                _ => unreachable!(),
            }
        }

        // Check for wickets in the outcome
        if let Some(wicket) = &ball.wicket {
            if matches!(wicket, Dismissal::RunOutNonStriker(_)) {
                self.batters[non_striker_idx].1.out = Some(wicket.clone());
            } else {
                striker_stats.out = Some(wicket.clone());
            }
        }

        // Replace batters if they've been made out
        if self.batters[self.batter_a].1.out.is_some() {
            // This may not be a valid index if the lineup is over
            self.batter_a = self.batters.len();
            if let Some(batter) = self.batting_order.next() {
                self.batters.push((batter, BatterInningsStats::new()));
            }
        }
        if self.batters[self.batter_b].1.out.is_some() {
            // This may not be a valid index if the lineup is over
            self.batter_b = self.batters.len();
            if let Some(batter) = self.batting_order.next() {
                self.batters.push((batter, BatterInningsStats::new()));
            }
        }

        if switch_striker {
            self.switch_striker();
        }
    }

    /// Print a summary table of the batting stats
    // TODO: Consider returning the table to allow printing to e.g. a file
    pub fn print_summary(&self) {
        use prettytable::{format::consts::*, Table};
        let mut table = Table::new();
        table.set_format(*FORMAT_NO_LINESEP_WITH_TITLE);
        // table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.set_titles(row!["Batter", "Wicket", "R (B)"]);
        for batter in &self.batters {
            table.add_row(row![
                batter.0.name,
                match &batter.1.out {
                    Some(wicket) => format!("{}", wicket),
                    None => "Not out".to_string(),
                },
                batter.1,
            ]);
        }
        table.printstd();
    }
}
