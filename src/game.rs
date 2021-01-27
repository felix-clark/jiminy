//! Description of the state and events of a match.
use crate::team::{BattingOrder, Bowlers, Team};
use crate::{form, player::Player};

use std::fmt::{self, Display};

pub struct GameState<'a> {
    /// The rules of the match
    form: form::Form,
    /// The home team
    team_a: &'a Team,
    /// The visiting team
    team_b: &'a Team,
    /// Current innings in-progress. Is None when the game is complete.
    current_innings_stats: Option<InningsStats<'a>>,
    /// Previous innings stats
    previous_innings: Vec<InningsStats<'a>>,
}

impl<'a> GameState<'a> {
    pub fn new(rules: form::Form, team_a: &'a Team, team_b: &'a Team) -> Self {
        let current_innings_stats = Some(InningsStats::new(team_a, team_b, rules.balls_per_over));
        Self {
            form: rules,
            team_a,
            team_b,
            current_innings_stats,
            previous_innings: Vec::new(),
        }
    }

    /// Whether the match is finished
    pub fn complete(&self) -> bool {
        // NOTE: There are other ways for a game to be finished than completion of all
        // innings.
        // e.g.: last batting team overtakes, time limit, team with no more
        // opportunities still has lower score, forfeture/abandonment, ...
        // Some of these are accounted for, but not all.
        self.current_innings_stats.is_none()
    }

    /// Batting team declares to complete their innings
    pub fn declare(&mut self) {
        self.new_innings();
    }

    /// Update the game state based on the outcome of a delivery
    pub fn update(&mut self, ball: &DeliveryOutcome) {
        let innings_stats = self.current_innings_stats.as_mut().expect("Match is over!");
        innings_stats.update(ball);

        // Check if we need to change to a new innings
        let mut new_innings = false;
        if innings_stats.all_out() {
            assert_eq!(innings_stats.wickets() + 1, self.form.batsmen_per_side);
            new_innings = true;
        }
        if let Some(opi) = self.form.overs_per_innings {
            if innings_stats.overs >= opi {
                new_innings = true;
            }
        }
        let batting_team = innings_stats.batting_team;
        let bowling_team = innings_stats.bowling_team;
        // If this is the last innings and the batting team caught up, end the match
        if self.previous_innings.len() + 1 == 2 * self.form.innings as usize
            && self.team_score(batting_team) > self.team_score(bowling_team)
        {
            new_innings = true;
        }
        if new_innings {
            self.new_innings();
        }
    }

    /// Initiate a new innings
    fn new_innings(&mut self) {
        let last_innings_stats = self
            .current_innings_stats
            .take()
            .expect("Must be a current innings");
        let last_batting_team = last_innings_stats.batting_team;
        let last_bowling_team = last_innings_stats.bowling_team;
        self.previous_innings.push(last_innings_stats);
        // If all innings have been played (or if the game is over), exit
        if self.previous_innings.len() >= 2 * self.form.innings as usize {
            return;
        }
        // Make the losing team go first regardless if they are losing by 150 or more
        // and both teams have had equal opportunities so far.
        let last_batting_runs = self.team_score(last_batting_team);
        let last_bowling_runs = self.team_score(last_bowling_team);

        // If the team just batting has run out of opportunities to overtake, the match
        // is called.
        if self.previous_innings.len() + 1 == 2 * self.form.innings as usize
            && last_batting_runs < last_bowling_runs
        {
            return;
        }

        let (next_batting_team, next_bowling_team) = if self.previous_innings.len() % 2 == 0
            && last_batting_runs + 150 <= last_bowling_runs
        {
            (last_batting_team, last_bowling_team)
        } else {
            (last_bowling_team, last_batting_team)
        };

        self.current_innings_stats = Some(InningsStats::new(
            next_batting_team,
            next_bowling_team,
            self.form.balls_per_over,
        ));
    }

    /// Returns the given team's current score
    pub fn team_score(&self, team: &Team) -> u16 {
        let mut score = self
            .previous_innings
            .iter()
            .filter_map(|st| {
                if st.batting_team == team {
                    Some(st.batting_stats.team_runs())
                } else {
                    None
                }
            })
            .sum::<u16>();
        if let Some(st) = &self.current_innings_stats {
            if st.batting_team == team {
                score += st.batting_stats.team_runs();
            }
        }
        score
    }

    /// Print a summary of each innings to stdout
    pub fn print_innings_summary(&self) {
        for innings in self.previous_innings.iter() {
            println!("\n{} innings:", innings.batting_team.name);
            innings.batting_stats.print_summary();
            innings
                .bowling_stats
                .print_summary(self.form.balls_per_over);
            println!("Total: {}/{}", innings.runs(), innings.wickets());
        }
        println!("\n{}: {}", self.team_a.name, self.team_score(self.team_a));
        println!("{}: {}", self.team_b.name, self.team_score(self.team_b));
    }
}

/// Methods of dismissal
#[derive(Clone)]
pub enum Dismissal {
    /// Legitimate delivery hits wicket and puts it down. (bowler)
    Bowled(String),
    /// Ball is hit in the air and caught in-bounds (caught, bowler)
    Caught(String, String),
    /// Leg before wicket: A delivery that would have hit the wickets instead first
    /// makes contact with the striker (not the bat). (bowler)
    Lbw(String),
    /// The striker is put out while running (fielder)
    // TODO: Consider not distinguishing these, but letting the simulation access both
    RunOutStriker(String),
    /// The only method by which the non-striker can be dismissed.
    RunOutNonStriker(String),
    /// The wicket-keeper puts down the wicket while the striker is out of the crease.
    /// Takes precedence over run-out. (bowler)
    Stumped(String),
    // TODO: rare dismissals
}

impl Display for Dismissal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Extra runs scored for a team that are not credited to an individual batter.
pub enum Extra {
    /// One penalty run. Additional runs can still be scored off a no-ball. These are
    /// counted against the bowler.
    NoBall,
    /// One penalty run is awarded if the ball is judged to not be hittable with a
    /// normal cricket swing. A wide that is also a no-ball is counted as a no-ball.
    /// Wides are counted against the bowler's stats.
    Wide,
    /// A bye in which the batsman does not make contact and the wicket is not made. Runs
    /// can be scored but this is often zero. Balls that make it to the boundary are
    /// scored as fours. Byes and leg byes can be scored from no-balls and wides. They
    /// are not counted against the bowler's stats.
    Bye(Runs),
    /// Similar to a bye, but with contact off the batter (not the bat) that is not LBW.
    /// They are not counted against the bowler's stats.
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
    /// Return the strike rate for the batter
    pub fn strike_rate(&self) -> f32 {
        (self.runs as f32) * 100. / (self.balls as f32)
    }
}

impl Default for BatterInningsStats {
    fn default() -> Self {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    batting_order: BattingOrder<'a>,
    // pub team: &'a Team,
    /// Individual batting stats
    batters: Vec<(&'a Player, BatterInningsStats)>,
    /// Extra runs awarded to the team this inning
    extras: u16,
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
            BatterInningsStats::default(),
        ));
        batters.push((
            batting_order.next().expect("Not enough batters in order"),
            BatterInningsStats::default(),
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
    fn all_out(&self) -> bool {
        let num_batters = self.batters.len();
        self.batter_a >= num_batters || self.batter_b >= num_batters
    }

    /// Return the total number of team runs
    pub fn team_runs(&self) -> u16 {
        let batter_runs = self.batters.iter().map(|(_, st)| st.runs).sum::<u16>();
        batter_runs + self.extras
    }

    /// Return the total number of wickets
    fn wickets(&self) -> u8 {
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
                self.batters.push((batter, BatterInningsStats::default()));
            }
        }
        if self.batters[self.batter_b].1.out.is_some() {
            // This may not be a valid index if the lineup is over
            self.batter_b = self.batters.len();
            if let Some(batter) = self.batting_order.next() {
                self.batters.push((batter, BatterInningsStats::default()));
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
        table.set_titles(row!["Batter", "Wicket", "R (B)", "4s", "6s", "SR"]);
        for batter in &self.batters {
            let batter_stats = &batter.1;
            table.add_row(row![
                batter.0.name,
                match &batter_stats.out {
                    Some(wicket) => format!("{}", wicket),
                    None => "Not out".to_string(),
                },
                batter_stats,
                batter_stats.fours,
                batter_stats.sixes,
                format!("{:.2}", batter_stats.strike_rate()),
            ]);
        }
        table.printstd();
    }
}

/// The bowling stats of a single bowler in a single innings
pub struct BowlerInningsStats {
    /// Number of balls bowled
    pub balls: u16,
    /// maiden overs
    pub maiden_overs: u16,
    /// Runs conceded
    pub runs: u16,
    /// Wickets taken
    pub wickets: u8,
    // TODO: consider tracking dots, 4s, and 6s
    /// Wides
    pub wides: u16,
    /// No-balls
    pub no_balls: u16,
}

impl BowlerInningsStats {
    /// Return the economy rate
    pub fn economy(&self, balls_per_over: u8) -> f32 {
        (self.runs as f32) * (balls_per_over as f32) / (self.balls as f32)
    }

    // NOTE: bowler average and strike rate are not reasonable stats to evaluate at the
    // level of a single innings
}

impl Default for BowlerInningsStats {
    /// Initialize all stats to zero
    fn default() -> Self {
        Self {
            balls: 0,
            maiden_overs: 0,
            runs: 0,
            wickets: 0,
            wides: 0,
            no_balls: 0,
        }
    }
}

impl Display for BowlerInningsStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: An alternative output would be X-Y-Z-W for overs, maidens, runs,
        // wickets respectively. Overs would require balls-per-over.
        write!(f, "{}/{}", self.wickets, self.runs)
    }
}

struct TeamBowlingInningsStats<'a> {
    /// Reference to team's bowling
    bowlers: Bowlers<'a>,
    /// Stats of individual bowlers
    bowler_stats: Vec<(&'a Player, BowlerInningsStats)>,
    /// Index of bowler that is currently bowling
    current_bowler_index: usize,
    /// Whether the current over is a maiden (so far)
    current_over_maiden: bool,
}

impl<'a> TeamBowlingInningsStats<'a> {
    /// Create a new team stats object for an innings
    pub fn new(team: &'a Team) -> Self {
        let mut bowlers = team.bowlers();
        let bowler_stats: Vec<(&Player, BowlerInningsStats)> = vec![(
            bowlers.next().expect("Could not get first bowler"),
            BowlerInningsStats::default(),
        )];
        Self {
            bowlers,
            bowler_stats,
            current_bowler_index: 0,
            current_over_maiden: true,
        }
    }

    /// Update the stats with a new delivery outcome
    pub fn update(&mut self, ball: &DeliveryOutcome) {
        let bowler_stats = &mut self.bowler_stats[self.current_bowler_index].1;

        if ball.legal() {
            bowler_stats.balls += 1;
        }
        let bowler_runs: u8 = ball.runs.runs()
            + ball
                .extras
                .iter()
                .filter_map(|x| match x {
                    Extra::NoBall | Extra::Wide => Some(x.runs()),
                    _ => None,
                })
                .sum::<u8>();
        if bowler_runs > 0 {
            self.current_over_maiden = false;
        }
        bowler_stats.runs += bowler_runs as u16;
        let wides = ball
            .extras
            .iter()
            .filter(|x| matches!(x, Extra::Wide))
            .count() as u16;
        bowler_stats.wides += wides;
        let no_balls = ball
            .extras
            .iter()
            .filter(|x| matches!(x, Extra::NoBall))
            .count() as u16;
        bowler_stats.no_balls += no_balls;
        if ball.wicket.is_some() {
            bowler_stats.wickets += 1;
        }
    }

    /// Indicate that there is a new over and switch bowlers.
    /// A bowler must finish an over unless incapacitated or suspended (we will ignore
    /// these cases for now).
    pub fn new_over(&mut self) {
        if self.current_over_maiden {
            self.bowler_stats[self.current_bowler_index].1.maiden_overs += 1;
        }
        self.current_over_maiden = true;

        let next_bowler: &Player = self.bowlers.next().expect("Could not get the next bowler");
        self.current_bowler_index = match self
            .bowler_stats
            .iter()
            .position(|(b, _)| b == &next_bowler)
        {
            Some(i) => i,
            None => {
                self.bowler_stats
                    .push((next_bowler, BowlerInningsStats::default()));
                self.bowler_stats.len() - 1
            }
        };
    }

    /// Print a summary table of the bowling stats
    // TODO: Consider returning the table to allow printing to e.g. a file
    pub fn print_summary(&self, balls_per_over: u8) {
        use prettytable::{format::consts::*, Table};
        let mut table = Table::new();
        table.set_format(*FORMAT_NO_LINESEP_WITH_TITLE);
        // table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.set_titles(row!["Bowler", "O", "M", "R", "W", "Econ"]);
        for bowler in &self.bowler_stats {
            let bowler_stats = &bowler.1;
            let overs_str = {
                let n_overs = bowler_stats.balls / balls_per_over as u16;
                let n_excess_balls = bowler_stats.balls % balls_per_over as u16;
                if n_excess_balls == 0 {
                    format!("{}", n_overs)
                } else {
                    format!("{}.{}", n_overs, n_excess_balls)
                }
            };
            table.add_row(row![
                bowler.0.name,
                overs_str,
                bowler_stats.maiden_overs,
                bowler_stats.runs,
                bowler_stats.wickets,
                format!("{:.2}", bowler_stats.economy(balls_per_over)),
            ]);
        }
        table.printstd();
    }
}

/// Collects and tracks stats in a given innings
struct InningsStats<'a> {
    batting_team: &'a Team,
    bowling_team: &'a Team,
    batting_stats: TeamBattingInningsStats<'a>,
    bowling_stats: TeamBowlingInningsStats<'a>,
    /// The number of overs that have been completed
    pub overs: u16,
    /// The number of legal balls delivered in the over
    pub balls: u8,
    /// The number of balls per over
    // TODO: Consider reference to Form?
    balls_per_over: u8,
}

impl<'a> InningsStats<'a> {
    pub fn new(batting_team: &'a Team, bowling_team: &'a Team, balls_per_over: u8) -> Self {
        Self {
            batting_team,
            bowling_team,
            batting_stats: TeamBattingInningsStats::new(batting_team),
            bowling_stats: TeamBowlingInningsStats::new(bowling_team),
            overs: 0,
            balls: 0,
            balls_per_over,
        }
    }

    /// Whether all (but one) batters have been made out. Indicates the innings must be
    /// complete.
    pub fn all_out(&self) -> bool {
        self.batting_stats.all_out()
    }

    /// Number of total runs scored
    pub fn runs(&self) -> u16 {
        self.batting_stats.team_runs()
    }

    /// Number of wickets taken
    pub fn wickets(&self) -> u8 {
        self.batting_stats.wickets()
    }

    /// Update the stats with a new delivery
    pub fn update(&mut self, ball: &DeliveryOutcome) {
        self.batting_stats.update(ball);
        self.bowling_stats.update(ball);
        if ball.legal() {
            self.balls += 1;
        }
        if self.balls >= self.balls_per_over {
            self.balls = 0;
            self.overs += 1;
            self.batting_stats.switch_striker();
            self.bowling_stats.new_over();
        }
    }
}
