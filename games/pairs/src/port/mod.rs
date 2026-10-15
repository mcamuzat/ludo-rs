//! Port (James Ernest): Pairs with a single winner. Take cards, or lock
//! your stack; a pair busts you; the highest stack scores the most.
//! Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{draw_logged, full_deck, has_rank, open_round, Opening, Rank, Supply};

pub struct Port;

/// [R] Cards burned after every shuffle.
pub const BURN: u8 = 5;
/// [R] Play to 21 game points.
pub const DEFAULT_TARGET: u8 = 21;
pub const MAX_TARGET: u8 = 99;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Game points to reach, capped at `MAX_TARGET`. `None` (or 0): 21.
    pub target: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Take a card.
    Hit,
    /// Take no more cards this round.
    Lock,
}

/// Where a seat stands in the round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Playing,
    Locked,
    Busted,
    /// Not in this round (a tie-break round among the leaders).
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Play,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new round among `seats`; `first` goes first.
    RoundStarted {
        round: u8,
        seats: Vec<Seat>,
        first: Seat,
    },
    Dealt {
        seat: Seat,
        card: Rank,
    },
    /// An opening tie-break card that would pair: discarded and replaced.
    Replaced {
        seat: Seat,
        card: Rank,
    },
    Busted {
        seat: Seat,
        card: Rank,
    },
    Locked {
        seat: Seat,
    },
    Reshuffled,
    /// Each seat's card total (`None`: busted or out of the round), the
    /// game points it earned, and the scores after the round.
    RoundScored {
        totals: Vec<Option<u32>>,
        points: Vec<u32>,
        scores: Vec<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub target: u8,
    pub round_no: u8,
    pub supply: Supply,
    /// Face-up cards of each seat this round.
    pub stacks: Vec<Vec<Rank>>,
    pub status: Vec<Status>,
    pub turn: Option<Seat>,
    pub phase: Phase,
    pub scores: Vec<u32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    pub fn total(&self, seat: Seat) -> u32 {
        self.stacks[seat.index()]
            .iter()
            .map(|c| u32::from(c.value()))
            .sum()
    }

    fn note(&mut self, log: Vec<Opening>) {
        self.events.extend(log.into_iter().map(|o| match o {
            Opening::Dealt { seat, card } => Event::Dealt { seat, card },
            Opening::Replaced { seat, card } => Event::Replaced { seat, card },
            Opening::Reshuffled => Event::Reshuffled,
        }));
    }

    /// [R] One card each to the players of the round; the lowest goes
    /// first, ties broken as in Pairs.
    fn start_round(&mut self, seats: Vec<Seat>) {
        self.round_no += 1;
        for seat in self.seats().collect::<Vec<_>>() {
            let stack = std::mem::take(&mut self.stacks[seat.index()]);
            self.supply.discard(stack);
            self.status[seat.index()] = if seats.contains(&seat) {
                Status::Playing
            } else {
                Status::Out
            };
        }
        let mut log = Vec::new();
        let first = open_round(&mut self.supply, &mut self.stacks, &seats, &mut log);
        self.events.push(Event::RoundStarted {
            round: self.round_no,
            seats,
            first,
        });
        self.note(log);
        self.turn = Some(first);
    }

    /// The next playing seat to the left of `seat` (itself last).
    fn next_playing(&self, seat: Seat) -> Option<Seat> {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| self.status[s.index()] == Status::Playing)
    }

    fn advance(&mut self, seat: Seat) {
        match self.next_playing(seat) {
            Some(next) => self.turn = Some(next),
            None => self.score_round(),
        }
    }

    /// [R] The highest total scores N game points (N: the players of the
    /// round), the next N-1, and so on; tied totals all score the higher
    /// amount; a bust scores nothing.
    pub fn round_points(totals: &[Option<u32>], n: u32) -> Vec<u32> {
        totals
            .iter()
            .map(|mine| match mine {
                None => 0,
                Some(mine) => {
                    let above = totals.iter().flatten().filter(|&t| t > mine).count() as u32;
                    n.saturating_sub(above)
                }
            })
            .collect()
    }

    fn score_round(&mut self) {
        self.turn = None;
        let totals: Vec<Option<u32>> = self
            .seats()
            .map(|s| (self.status[s.index()] == Status::Locked).then(|| self.total(s)))
            .collect();
        let n = self.status.iter().filter(|&&st| st != Status::Out).count() as u32;
        let points = Self::round_points(&totals, n);
        for (score, p) in self.scores.iter_mut().zip(&points) {
            *score += p;
        }
        self.events.push(Event::RoundScored {
            totals,
            points,
            scores: self.scores.clone(),
        });
        // [R] Play to the target; leaders tied past it play one more round
        // among themselves.
        let top = self.scores.iter().copied().max().unwrap_or(0);
        if top < u32::from(self.target) {
            self.start_round(self.seats().collect());
            return;
        }
        let leaders: Vec<Seat> = self
            .seats()
            .filter(|s| self.scores[s.index()] == top)
            .collect();
        if leaders.len() == 1 {
            self.phase = Phase::Over;
        } else {
            self.start_round(leaders);
        }
    }

    fn hit(&mut self, seat: Seat) {
        let mut log = Vec::new();
        let card = draw_logged(&mut self.supply, &mut log);
        self.note(log);
        let Some(card) = card else {
            return;
        };
        let i = seat.index();
        let paired = has_rank(&self.stacks[i], card);
        self.stacks[i].push(card);
        if paired {
            self.status[i] = Status::Busted;
            self.events.push(Event::Busted { seat, card });
        } else {
            self.events.push(Event::Dealt { seat, card });
        }
        self.advance(seat);
    }
}

impl Game for Port {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-port",
            name: "Port",
            min_players: 2,
            max_players: 7,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let target = config
            .target
            .filter(|&t| t > 0)
            .map_or(DEFAULT_TARGET, |t| t.min(MAX_TARGET));
        let n = usize::from(players);
        let mut s = State {
            players,
            target,
            round_no: 0,
            supply: Supply::new(full_deck(), BURN, seed),
            stacks: vec![Vec::new(); n],
            status: vec![Status::Playing; n],
            turn: None,
            phase: Phase::Play,
            scores: vec![0; n],
            events: Vec::new(),
        };
        s.start_round((0..players).map(Seat).collect());
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
        // ❓2: with no card left anywhere to draw, only locking.
        if s.supply.can_draw() {
            vec![Action::Hit, Action::Lock]
        } else {
            vec![Action::Lock]
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Il n'y a plus de carte à prendre."));
        }
        s.events.clear();
        match action {
            Action::Hit => s.hit(seat),
            Action::Lock => {
                s.status[seat.index()] = Status::Locked;
                s.events.push(Event::Locked { seat });
                s.advance(seat);
            }
        }
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
