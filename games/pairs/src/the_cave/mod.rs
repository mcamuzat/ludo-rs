//! The Cave (James Ernest, Cheapass Games 2018): play cards into a cave of
//! five face-up cards to capture its monsters. Rules and their sources:
//! RULES.md, where [C] is the monthly sheet.

pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, Rank};

pub struct TheCave;

/// [C] Cards face up in the Cave after each refill.
pub const CAVE_SIZE: usize = 5;

/// [C] Cards dealt to each player: 7, 6, 5, 5, 4 for 2 to 6 players.
pub fn hand_size(players: u8) -> usize {
    match players {
        0..=2 => 7,
        3 => 6,
        4 | 5 => 5,
        _ => 4,
    }
}

/// [C] What `card` captures: the cards of the rank closest beneath it, or,
/// when none is lower, every card of the highest rank (even its own). ❓
/// An empty Cave gives nothing. Sorted.
pub fn capture(cave: &[Rank], card: Rank) -> Vec<Rank> {
    let target = cave
        .iter()
        .copied()
        .filter(|&c| c < card)
        .max()
        .or_else(|| cave.iter().copied().max());
    target.map_or_else(Vec::new, |t| {
        cave.iter().copied().filter(|&c| c == t).collect()
    })
}

/// [C] For each rank, whoever captured the most cards of it scores one
/// point per such card; ties each score in full.
pub fn round_points(captured: &[Vec<Rank>]) -> Vec<u16> {
    let counts: Vec<[u8; 10]> = captured
        .iter()
        .map(|c| crate::deck::count_by_rank(c))
        .collect();
    let mut points = vec![0u16; captured.len()];
    for rank in 0..10 {
        let most = counts.iter().map(|c| c[rank]).max().unwrap_or(0);
        if most == 0 {
            continue;
        }
        for (p, c) in points.iter_mut().zip(&counts) {
            if c[rank] == most {
                *p += u16::from(most);
            }
        }
    }
    points
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Play { card: Rank },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Play,
    Over,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Dealt {
        round: u8,
        dealer: Seat,
        first: Seat,
    },
    Played {
        seat: Seat,
        card: Rank,
        captured: Vec<Rank>,
    },
    Refilled {
        cards: Vec<Rank>,
    },
    RoundOver {
        points: Vec<u16>,
        scores: Vec<u16>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub round_no: u8,
    /// For each round's shuffle. Never in a view.
    pub seed: u64,
    pub dealer: Seat,
    /// Face down; the top is the last element. Never in a view.
    pub deck: Vec<Rank>,
    pub cave: Vec<Rank>,
    pub hands: Vec<Vec<Rank>>,
    /// The cards each seat captured this round, face up.
    pub captured: Vec<Vec<Rank>>,
    pub scores: Vec<u16>,
    pub phase: Phase,
    pub turn: Seat,
    /// Since the last action.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl State {
    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// The distinct ranks in `seat`'s hand, lowest first.
    pub fn playable(&self, seat: Seat) -> Vec<Rank> {
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        cards
    }

    /// [C] Deal the hands, then five cards face up for the Cave; the player
    /// on the dealer's left starts.
    fn deal(&mut self, dealer: Seat) {
        let n = usize::from(self.players);
        self.round_no += 1;
        self.deck = full_deck();
        self.deck.shuffle(&mut ChaCha8Rng::seed_from_u64(mix(
            self.seed,
            self.round_no,
        )));
        self.hands = vec![Vec::new(); n];
        for _ in 0..hand_size(self.players) {
            for hand in &mut self.hands {
                hand.extend(self.deck.pop());
            }
        }
        self.cave.clear();
        self.captured = vec![Vec::new(); n];
        self.dealer = dealer;
        self.turn = self.next(dealer);
        self.events.push(Event::Dealt {
            round: self.round_no,
            dealer,
            first: self.turn,
        });
        self.refill();
    }

    /// [C] Back to five cards from the deck; an empty deck refills nothing.
    fn refill(&mut self) {
        let mut cards = Vec::new();
        while self.cave.len() < CAVE_SIZE {
            let Some(card) = self.deck.pop() else { break };
            self.cave.push(card);
            cards.push(card);
        }
        if !cards.is_empty() {
            self.events.push(Event::Refilled { cards });
        }
    }

    fn play(&mut self, seat: Seat, card: Rank) {
        let hand = &mut self.hands[seat.index()];
        if let Some(at) = hand.iter().position(|&c| c == card) {
            hand.remove(at);
        }
        let taken = capture(&self.cave, card);
        if let Some(&rank) = taken.first() {
            self.cave.retain(|&c| c != rank);
        }
        self.captured[seat.index()].extend(&taken);
        self.cave.push(card);
        self.events.push(Event::Played {
            seat,
            card,
            captured: taken,
        });
        self.refill();
        // [C] Play until everyone has emptied their hand (hands are equal).
        if self.hands.iter().all(Vec::is_empty) {
            self.end_round();
        } else {
            self.turn = self.next(seat);
        }
    }

    /// [C] Score the majorities, then one round per player with the deal
    /// passing left.
    fn end_round(&mut self) {
        let points = round_points(&self.captured);
        for (score, p) in self.scores.iter_mut().zip(&points) {
            *score += p;
        }
        self.events.push(Event::RoundOver {
            points,
            scores: self.scores.clone(),
        });
        if self.round_no >= self.players {
            self.phase = Phase::Over;
        } else {
            let dealer = self.next(self.dealer);
            self.deal(dealer);
        }
    }
}

impl Game for TheCave {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-the-cave",
            name: "The Cave",
            min_players: 2,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            round_no: 0,
            seed,
            dealer: Seat(0),
            deck: Vec::new(),
            cave: Vec::new(),
            hands: vec![Vec::new(); n],
            captured: vec![Vec::new(); n],
            scores: vec![0; n],
            phase: Phase::Play,
            turn: Seat(0),
            events: Vec::new(),
        };
        // ❓ The first dealer is drawn from the seed.
        let dealer = Seat((seed % u64::from(players.max(1))) as u8);
        s.deal(dealer);
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Play => Pending::one(s.turn),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Play || s.turn != seat {
            return Vec::new();
        }
        s.playable(seat)
            .into_iter()
            .map(|card| Action::Play { card })
            .collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != seat {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Tu n'as pas cette carte."));
        }
        s.events.clear();
        let Action::Play { card } = *action;
        s.play(seat, card);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// ❓ Equal totals share a rank.
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
