//! Description of the state and events of a match.
use crate::{
    conditions::{Conditions, Weather},
    error::{Error, Result},
    form,
    model::PlayerRating,
    player::{Player, PlayerDb, PlayerId},
    team::Team,
};
pub mod stats;
use stats::InningsStats;

use std::fmt::{self, Display};

/// Tracks the state of an ongoing match
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
    /// Other conditions
    conditions: Conditions,
}

/// The snapshot at a moment (e.g. striker, bowler, non-striker, fielders...)
pub struct GameSnapshot<'a, R>
where
    R: PlayerRating,
{
    pub bowler: &'a Player<R>,
    pub striker: &'a Player<R>,
    pub non_striker: &'a Player<R>,
    pub conditions: Conditions,
}

impl<'a> GameState<'a> {
    pub fn new(rules: form::Form, team_a: &'a Team, team_b: &'a Team) -> Result<Self> {
        let current_innings_stats = Some(InningsStats::new(team_a, team_b, rules.balls_per_over)?);
        let ball = rules.new_ball();
        Ok(Self {
            form: rules,
            team_a,
            team_b,
            current_innings_stats,
            previous_innings: Vec::new(),
            conditions: Conditions {
                ball,
                weather: Weather {},
            },
        })
    }

    // TODO: might need to constrain the db and snapshot references to distinguish them from the
    // lifetime of this GameState
    pub fn snapshot<'b, R>(&self, db: &'b PlayerDb<R>) -> Result<GameSnapshot<'b, R>>
    where
        R: PlayerRating,
    {
        let bowler_id = self.bowler().ok_or(Error::MatchComplete)?;
        let striker_id = self.striker().ok_or(Error::MatchComplete)?;
        let non_striker_id = self.non_striker().ok_or(Error::MatchComplete)?;
        let bowler = db
            .get(bowler_id)
            .ok_or(Error::PlayerNotFound(bowler_id))?;
        let striker = db
            .get(striker_id)
            .ok_or(Error::PlayerNotFound(striker_id))?;
        let non_striker = db
            .get(non_striker_id)
            .ok_or(Error::PlayerNotFound(non_striker_id))?;
        let conditions = self.conditions.clone();
        Ok(GameSnapshot {
            bowler,
            striker,
            non_striker,
            conditions,
        })
    }

    /// Get the current bowler
    fn bowler(&self) -> Option<PlayerId> {
        self.current_innings_stats
            .as_ref()
            .map(|st| st.bowling_stats.current_bowler())
    }
    fn striker(&self) -> Option<PlayerId> {
        self.current_innings_stats
            .as_ref()
            .map(|st| st.batting_stats.striker())
    }
    fn non_striker(&self) -> Option<PlayerId> {
        self.current_innings_stats
            .as_ref()
            .map(|st| st.batting_stats.non_striker())
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
    pub fn declare(&mut self) -> Result<()> {
        self.new_innings()
    }

    /// Update the game state based on the outcome of a delivery
    pub fn update(&mut self, ball: &DeliveryOutcome) -> Result<()> {
        self.conditions.ball.update(ball);

        let innings_stats = self
            .current_innings_stats
            .as_mut()
            .ok_or(Error::MatchComplete)?;
        innings_stats.update(ball)?;

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
            self.new_innings()?;
        }
        Ok(())
    }

    /// Initiate a new innings
    fn new_innings(&mut self) -> Result<()> {
        let last_innings_stats = self
            .current_innings_stats
            .take()
            .ok_or(Error::MatchComplete)?;
        let last_batting_team = last_innings_stats.batting_team;
        let last_bowling_team = last_innings_stats.bowling_team;
        self.previous_innings.push(last_innings_stats);
        // If all innings have been played (or if the game is over), exit
        if self.previous_innings.len() >= 2 * self.form.innings as usize {
            return Ok(());
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
            return Ok(());
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
        )?);
        Ok(())
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
    pub fn print_innings_summary(&self) -> Result<()> {
        for innings in self.previous_innings.iter() {
            println!("\n{} innings:", innings.batting_team.name);
            innings.batting_stats.print_summary(innings.batting_team)?;
            innings
                .bowling_stats
                .print_summary(innings.bowling_team, self.form.balls_per_over)?;
            println!("Total: {}/{}", innings.runs(), innings.wickets());
        }
        println!("\n{}: {}", self.team_a.name, self.team_score(self.team_a));
        println!("{}: {}", self.team_b.name, self.team_score(self.team_b));
        Ok(())
    }
}

/// Methods of dismissal
/// TODO: Consider holding PlayerId instead of name. The means we need another struct created with
/// a PlayerDb to implement Display.
#[derive(Clone)]
pub enum Dismissal {
    /// Legitimate delivery hits wicket and puts it down.
    Bowled { bowler: String },
    /// Ball is hit in the air and caught in-bounds
    Caught { caught: String, bowler: String },
    /// Leg before wicket: A delivery that would have hit the wickets instead first
    /// makes contact with the striker (not the bat). (bowler)
    Lbw { bowler: String },
    /// The striker is put out while running (fielder)
    // TODO: Consider not distinguishing these, but letting the simulation access both
    RunOutStriker(String),
    /// The only method by which the non-striker can be dismissed.
    RunOutNonStriker(String),
    /// The wicket-keeper puts down the wicket while the striker is out of the crease.
    /// Takes precedence over run-out.
    Stumped { keeper: String },
    // TODO: rare dismissals
}

impl Display for Dismissal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Dismissal::*;
        match &self {
            Bowled { bowler } => write!(f, "b {}", bowler),
            Caught { caught, bowler } => write!(f, "c {} b {}", caught, bowler),
            Lbw { bowler } => write!(f, "lbw b {}", bowler),
            RunOutStriker(fielder) | RunOutNonStriker(fielder) => write!(f, "runout ({})", fielder),
            Stumped { keeper } => write!(f, "st {}", keeper),
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
    /// can be scored but this is often zero. Balls that make it to the boundary are scored as
    /// fours. Byes and leg byes can be scored from no-balls and wides. Neither are counted against
    /// the bowler's stats, although Byes are counted against the wicket-keeper.
    Bye(Runs),
    /// Similar to a bye, but with contact off the batter (not the bat) that is not LBW.
    /// They are not counted against the bowler's or wicket keeper's stats.
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
    pub wicket: Option<(PlayerId, Dismissal)>,
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

    // TODO: These should take the bowler ID and not just the name. This will require hooking up to
    // a PlayerDb to display.
    pub fn bowled(striker_id: PlayerId, bowler_name: &str) -> Self {
        Self {
            wicket: Some((
                striker_id,
                Dismissal::Bowled {
                    bowler: bowler_name.to_string(),
                },
            )),
            ..Default::default()
        }
    }

    pub fn caught(striker_id: PlayerId, bowler_name: &str, catcher_name: &str) -> Self {
        Self {
            wicket: Some((
                striker_id,
                Dismissal::Caught {
                    caught: catcher_name.to_string(),
                    bowler: bowler_name.to_string(),
                },
            )),
            ..Default::default()
        }

    }

    pub fn lbw(striker_id: PlayerId, bowler_name: &str) -> Self {
        Self {
            wicket: Some((
                striker_id,
                Dismissal::Lbw {
                    bowler: bowler_name.to_string(),
                },
            )),
            ..Default::default()
        }
    }

    pub fn dot() -> Self {
        Self::default()
    }

    pub fn four() -> Self {
        Self {
            runs: Runs::Four,
            ..Default::default()
        }
    }

    pub fn six() -> Self {
        Self {
            runs: Runs::Six,
            ..Default::default()
        }
    }

    pub fn running(runs: u8) -> Self {
        Self {
            runs: Runs::Running(runs),
            ..Default::default()
        }
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
