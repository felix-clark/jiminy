//! Player and team stats from a match

use super::{DeliveryOutcome, Dismissal, Extra, Runs};
use crate::{
    error::{Error, Result},
    player::PlayerId,
    team::{BattingOrder, Bowlers, Team},
};
use std::fmt::{self, Display};

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

pub(crate) struct TeamBattingInningsStats {
    /// Reference to the team's lineup
    batting_order: BattingOrder,
    // pub team: &'a Team,
    /// Individual batting stats
    batters: Vec<(PlayerId, BatterInningsStats)>,
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

impl TeamBattingInningsStats {
    /// Create a new team stats object for a fresh innings
    pub fn new(team: &Team) -> Result<Self> {
        let mut batting_order = team.batting_order();
        let mut batters = Vec::new();
        batters.push((
            batting_order
                .next()
                .ok_or_else(|| Error::MissingData("No first batter".into()))?,
            BatterInningsStats::default(),
        ));
        batters.push((
            batting_order
                .next()
                .ok_or_else(|| Error::MissingData("No second batter".into()))?,
            BatterInningsStats::default(),
        ));
        Ok(Self {
            batting_order,
            batters,
            extras: 0,
            batter_a: 0,
            batter_b: 1,
            striker_a: true,
        })
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
    // pub fn striker(&self) -> Option<PlayerId> {
    pub fn striker(&self) -> PlayerId {
        let striker_idx = if self.striker_a {
            self.batter_a
        } else {
            self.batter_b
        };
        assert!(
            striker_idx < self.batters.len(),
            "Innings is over, can't get striker"
        );
        self.batters[striker_idx].0
    }

    /// Returns a reference to the current non-striker
    pub fn non_striker(&self) -> PlayerId {
        let non_striker_idx = if self.striker_a {
            self.batter_b
        } else {
            self.batter_a
        };
        assert!(
            non_striker_idx < self.batters.len(),
            "Innings is over, can't get striker"
        );
        self.batters[non_striker_idx].0
    }

    /// Update the stats of a batter based on a delivery outcome
    pub fn update(&mut self, ball: &DeliveryOutcome) -> Result<()> {
        let striker_idx = if self.striker_a {
            self.batter_a
        } else {
            self.batter_b
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
        drop(&striker_stats);
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
        if let Some((out_id, wicket)) = &ball.wicket {
            let out_stats = self
                .batters
                .iter_mut()
                .find(|(id, _)| id == out_id)
                .ok_or_else(|| Error::PlayerNotFound(*out_id))?;
            out_stats.1.out = Some(wicket.clone());

            //if matches!(wicket, Dismissal::RunOutNonStriker(_)) {
            //self.batters[non_striker_idx].1.out = Some(wicket.clone());
            //} else {
            //striker_stats.out = Some(wicket.clone());
            //}
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
        Ok(())
    }

    /// Print a summary table of the batting stats
    // TODO: Consider returning the table to allow printing to e.g. a file
    pub fn print_summary(&self, team: &Team) -> Result<()> {
        use prettytable::{format::consts::*, Table};
        let mut table = Table::new();
        table.set_format(*FORMAT_NO_LINESEP_WITH_TITLE);
        // table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.set_titles(row!["Batter", "Wicket", "R (B)", "4s", "6s", "SR"]);
        for batter in &self.batters {
            let batter_stats = &batter.1;
            table.add_row(row![
                team.get_name(batter.0)
                    .ok_or_else(|| Error::PlayerNotFound(batter.0))?,
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
        Ok(())
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

pub(crate) struct TeamBowlingInningsStats {
    /// Reference to team's bowling
    bowlers: Bowlers,
    /// Stats of individual bowlers
    bowler_stats: Vec<(PlayerId, BowlerInningsStats)>,
    /// Index of bowler that is currently bowling
    current_bowler_index: usize,
    /// Whether the current over is a maiden (so far)
    current_over_maiden: bool,
}

impl TeamBowlingInningsStats {
    /// Create a new team stats object for an innings
    pub fn new(team: &Team) -> Result<Self> {
        let mut bowlers = team.bowlers();
        let bowler_stats: Vec<(PlayerId, BowlerInningsStats)> = vec![(
            bowlers
                .next()
                .ok_or_else(|| Error::MissingData("Could not get first bowler".into()))?,
            BowlerInningsStats::default(),
        )];
        Ok(Self {
            bowlers,
            bowler_stats,
            current_bowler_index: 0,
            current_over_maiden: true,
        })
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
    pub fn new_over(&mut self) -> Result<()> {
        if self.current_over_maiden {
            self.bowler_stats[self.current_bowler_index].1.maiden_overs += 1;
        }
        self.current_over_maiden = true;

        let next_bowler: PlayerId = self
            .bowlers
            .next()
            .ok_or_else(|| Error::MissingData("Could not get next bowler".into()))?;
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
        Ok(())
    }

    /// Returns a reference to the current bowler
    pub fn current_bowler(&self) -> PlayerId {
        self.bowler_stats[self.current_bowler_index].0
    }

    /// Print a summary table of the bowling stats
    // TODO: Consider returning the table to allow printing to e.g. a file
    pub fn print_summary(&self, team: &Team, balls_per_over: u8) -> Result<()> {
        use prettytable::{format::consts::*, Table};
        let mut table = Table::new();
        table.set_format(*FORMAT_NO_LINESEP_WITH_TITLE);
        // table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.set_titles(row!["Bowler", "O", "M", "R", "W", "Econ"]);
        for bowler in &self.bowler_stats {
            let (bowler_id, bowler_stats) = bowler;
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
                team.get_name(*bowler_id)
                    .ok_or_else(|| Error::PlayerNotFound(*bowler_id))?,
                overs_str,
                bowler_stats.maiden_overs,
                bowler_stats.runs,
                bowler_stats.wickets,
                format!("{:.2}", bowler_stats.economy(balls_per_over)),
            ]);
        }
        table.printstd();
        Ok(())
    }
}

/// Collects and tracks stats in a given innings
pub(crate) struct InningsStats<'a> {
    pub batting_team: &'a Team,
    pub bowling_team: &'a Team,
    pub batting_stats: TeamBattingInningsStats,
    pub bowling_stats: TeamBowlingInningsStats,
    /// The number of overs that have been completed
    pub overs: u16,
    /// The number of legal balls delivered in the over
    pub balls: u8,
    /// The number of balls per over
    // TODO: Consider reference to Form?
    balls_per_over: u8,
}

impl<'a> InningsStats<'a> {
    pub fn new(batting_team: &'a Team, bowling_team: &'a Team, balls_per_over: u8) -> Result<Self> {
        Ok(Self {
            batting_team,
            bowling_team,
            batting_stats: TeamBattingInningsStats::new(batting_team)?,
            bowling_stats: TeamBowlingInningsStats::new(bowling_team)?,
            overs: 0,
            balls: 0,
            balls_per_over,
        })
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
    pub fn update(&mut self, ball: &DeliveryOutcome) -> Result<()> {
        self.batting_stats.update(ball)?;
        self.bowling_stats.update(ball);
        if ball.legal() {
            self.balls += 1;
        }
        if self.balls >= self.balls_per_over {
            self.balls = 0;
            self.overs += 1;
            self.batting_stats.switch_striker();
            self.bowling_stats.new_over()?;
        }
        Ok(())
    }
}
