//! Pendleton (James Ernest), once "Five Cards": two players draw and play
//! cards on their side of five target cards. Rules and their sources:
//! RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{ColumnView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply};

pub struct Pendleton;

/// [R] Five target cards.
pub const COLUMNS: usize = 5;
/// [R] The first player to 100 points wins.
pub const DEFAULT_TARGET: u16 = 100;
pub const MAX_TARGET: u16 = 1000;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Points to reach, capped at `MAX_TARGET`. `None` (or 0): 100.
    pub target: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play the drawn card on one's side of `column` (0 = lowest target).
    Play { column: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Play,
    Over,
}

/// One column at the end of a round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnScore {
    pub target: Rank,
    /// Points each seat played there.
    pub played: Vec<u32>,
    /// Points each seat scores for it.
    pub scored: Vec<u32>,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new round: its targets, and who goes first.
    RoundStarted { targets: Vec<Rank>, first: Seat },
    Played {
        seat: Seat,
        card: Rank,
        column: u8,
        target: Rank,
    },
    /// The round is over: `seat` drew `card`, which no column accepts
    /// (`None`: the draw pile ran out, ❓4).
    RoundScored {
        seat: Seat,
        card: Option<Rank>,
        columns: Vec<ColumnScore>,
        /// Points each seat scored this round.
        round: Vec<u32>,
        /// Totals after the round.
        totals: Vec<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub target_score: u16,
    /// The current round, from 1.
    pub round_no: u8,
    /// For each round's deal. Never in a view.
    pub seed: u64,
    pub supply: Supply,
    /// Five distinct ranks, lowest first.
    pub targets: Vec<Rank>,
    /// `played[seat][column]`: the cards on that side of that column.
    pub played: Vec<Vec<Vec<Rank>>>,
    /// The card the player to move has drawn, face up.
    pub drawn: Option<Rank>,
    pub turn: Option<Seat>,
    /// Who went first this round.
    pub first: Seat,
    pub phase: Phase,
    pub scores: Vec<u32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl State {
    fn other(seat: Seat) -> Seat {
        Seat(1 - seat.0)
    }

    /// [R] A card may go to a column holding no card of its rank: not the
    /// target, not a card on either side.
    pub fn accepts(&self, column: usize, card: Rank) -> bool {
        self.targets[column] != card
            && self
                .played
                .iter()
                .all(|side| !has_rank(&side[column], card))
    }

    pub fn open_columns(&self, card: Rank) -> Vec<usize> {
        (0..self.targets.len())
            .filter(|&c| self.accepts(c, card))
            .collect()
    }

    /// Points `seat` has played in `column`.
    pub fn points(&self, seat: Seat, column: usize) -> u32 {
        self.played[seat.index()][column]
            .iter()
            .map(|c| u32::from(c.value()))
            .sum()
    }

    /// [R] Whoever played the most points in a column scores its target;
    /// on a tie each scores half, rounded down. ❓3: a column nobody
    /// played in scores nothing.
    pub fn column_score(&self, column: usize) -> ColumnScore {
        let target = self.targets[column];
        let played: Vec<u32> = (0..2).map(|i| self.points(Seat(i), column)).collect();
        let value = u32::from(target.value());
        let scored = match played[0].cmp(&played[1]) {
            std::cmp::Ordering::Greater => vec![value, 0],
            std::cmp::Ordering::Less => vec![0, value],
            std::cmp::Ordering::Equal if played[0] == 0 => vec![0, 0],
            std::cmp::Ordering::Equal => vec![value / 2; 2],
        };
        ColumnScore {
            target,
            played,
            scored,
        }
    }

    /// [R] Five cards of distinct ranks, sorted; the duplicates dealt
    /// meanwhile go back and the deck is reshuffled.
    fn start_round(&mut self, first: Seat) {
        self.round_no += 1;
        let stream = mix(self.seed, self.round_no);
        let mut deal = Supply::new(full_deck(), 0, stream);
        let mut targets = Vec::new();
        while targets.len() < COLUMNS {
            match deal.draw() {
                Some(card) if !targets.contains(&card) => targets.push(card),
                Some(_) => {}
                None => break,
            }
        }
        targets.sort_unstable();
        let mut rest = full_deck();
        for card in &targets {
            if let Some(at) = rest.iter().position(|c| c == card) {
                rest.remove(at);
            }
        }
        self.supply = Supply::new(rest, 0, stream.rotate_left(17));
        self.played = vec![vec![Vec::new(); targets.len()]; 2];
        self.events.push(Event::RoundStarted {
            targets: targets.clone(),
            first,
        });
        self.targets = targets;
        self.first = first;
        self.begin_turn(first);
    }

    /// [R] Draw a card; a card no column accepts ends the round.
    fn begin_turn(&mut self, seat: Seat) {
        let card = self.supply.draw();
        match card {
            Some(card) if !self.open_columns(card).is_empty() => {
                self.drawn = Some(card);
                self.turn = Some(seat);
            }
            _ => self.end_round(seat, card),
        }
    }

    fn end_round(&mut self, seat: Seat, card: Option<Rank>) {
        let columns: Vec<ColumnScore> = (0..self.targets.len())
            .map(|c| self.column_score(c))
            .collect();
        let round: Vec<u32> = (0..2)
            .map(|i| columns.iter().map(|c| c.scored[i]).sum())
            .collect();
        for (total, got) in self.scores.iter_mut().zip(&round) {
            *total += got;
        }
        self.events.push(Event::RoundScored {
            seat,
            card,
            columns,
            round: round.clone(),
            totals: self.scores.clone(),
        });
        self.drawn = None;
        self.turn = None;
        let target = u32::from(self.target_score);
        // ❓1: the game ends once someone has reached the target with
        // more points than the other.
        if self.scores.iter().any(|&s| s >= target) && self.scores[0] != self.scores[1] {
            self.phase = Phase::Over;
            return;
        }
        // [R] Whoever scored fewer points goes first; ❓2 on a tie, the
        // player who went second.
        let first = match round[0].cmp(&round[1]) {
            std::cmp::Ordering::Less => Seat(0),
            std::cmp::Ordering::Greater => Seat(1),
            std::cmp::Ordering::Equal => Self::other(self.first),
        };
        self.start_round(first);
    }

    fn play(&mut self, seat: Seat, column: usize) {
        let Some(card) = self.drawn.take() else {
            return;
        };
        self.played[seat.index()][column].push(card);
        self.events.clear();
        self.events.push(Event::Played {
            seat,
            card,
            column: column as u8,
            target: self.targets[column],
        });
        self.begin_turn(Self::other(seat));
    }
}

impl Game for Pendleton {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-pendleton",
            name: "Pendleton",
            min_players: 2,
            max_players: 2,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, _players: u8, seed: u64) -> State {
        let target_score = config
            .target
            .filter(|&t| t > 0)
            .map_or(DEFAULT_TARGET, |t| t.min(MAX_TARGET));
        let mut s = State {
            target_score,
            round_no: 0,
            seed,
            supply: Supply::new(Vec::new(), 0, seed),
            targets: Vec::new(),
            played: Vec::new(),
            drawn: None,
            turn: None,
            first: Seat(0),
            phase: Phase::Play,
            scores: vec![0, 0],
            events: Vec::new(),
        };
        // [R] A random player goes first.
        let first = Seat((ChaCha8Rng::seed_from_u64(seed).next_u32() % 2) as u8);
        s.start_round(first);
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Play || s.turn != Some(seat) {
            return Vec::new();
        }
        let Some(card) = s.drawn else {
            return Vec::new();
        };
        s.open_columns(card)
            .into_iter()
            .map(|c| Action::Play { column: c as u8 })
            .collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new(
                "Cette colonne contient déjà une carte de ce rang.",
            ));
        }
        let Action::Play { column } = *action;
        s.play(seat, usize::from(column));
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        let ranks = s
            .scores
            .iter()
            .map(|&mine| s.scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
