//! Blackstone (James Ernest): stones from a bag. The 10s are black stones,
//! everything else is white; whoever collects two black stones loses.
//! Take at least as many cards as the player before you, or fold. Rules
//! and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, Rank, Supply};

pub struct Blackstone;

/// [R] Each player antes 5 chips, kept in front of them.
pub const ANTE: u32 = 5;
/// [R] The 10s are black stones.
pub const BLACK: u8 = 10;
/// ❓1
pub const DEFAULT_HANDS: u8 = 5;
pub const MAX_HANDS: u8 = 15;

pub fn is_black(card: Rank) -> bool {
    card.value() == BLACK
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Hands in a game, capped at `MAX_HANDS`. `None` (or 0): 5.
    pub hands: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Take `count` cards, declared before any is revealed.
    Take {
        count: u8,
    },
    Fold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    In,
    Folded,
    /// Collected two black stones.
    Lost,
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
    /// A new hand: the opening cards of each seat, who goes first, and how
    /// many extra black stones went back into the deck.
    Opened {
        hand: u8,
        cards: Vec<Vec<Rank>>,
        first: Seat,
        returned: u8,
    },
    /// `seat` declared `count` and drew `cards` (fewer when it lost).
    Took {
        seat: Seat,
        count: u8,
        cards: Vec<Rank>,
    },
    Folded {
        seat: Seat,
        paid: u32,
    },
    /// `seat` collected two black stones: its ante plus one chip per white
    /// stone.
    Lost {
        seat: Seat,
        paid: u32,
    },
    /// The survivors take back their antes and split the pot, `share`
    /// each; `left` chips stay for the next hand.
    PotWon {
        seats: Vec<Seat>,
        share: u32,
        left: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub hands_total: u8,
    pub hand_no: u8,
    /// For each hand's shuffle. Never in a view.
    pub seed: u64,
    pub supply: Supply,
    /// The white stones dealt this hand, in the middle.
    pub whites: Vec<Rank>,
    /// Black stones each seat holds.
    pub blacks: Vec<u8>,
    pub status: Vec<Status>,
    /// The fewest cards the next player may take.
    pub min_take: u8,
    pub turn: Option<Seat>,
    pub phase: Phase,
    pub pot: u32,
    /// Each seat's balance since the start, antes in front counted as
    /// spent; with the antes in front and the pot, they sum to zero.
    pub chips: Vec<i32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

/// One independent stream per hand.
fn mix(seed: u64, hand: u8) -> u64 {
    seed ^ u64::from(hand).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    pub fn alive(&self) -> Vec<Seat> {
        self.seats()
            .filter(|s| self.status[s.index()] == Status::In)
            .collect()
    }

    /// Antes still in front of the players.
    pub fn in_front(&self) -> u32 {
        self.alive().len() as u32 * ANTE
    }

    /// [R] Folding costs half the white stones dealt, rounded up (on top
    /// of the ante, which goes to the pot).
    pub fn fold_cost(&self) -> u32 {
        (self.whites.len() as u32).div_ceil(2)
    }

    /// [R] Everyone antes; one face-up card each, the lowest first. Tied
    /// players get more cards; pairs of white stones stay. Extra black
    /// stones go back into the deck, which is reshuffled. White stones go
    /// to the middle; players keep their black stones.
    fn start_hand(&mut self) {
        self.hand_no += 1;
        let n = usize::from(self.players);
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.hand_no));
        self.whites.clear();
        self.blacks = vec![0; n];
        self.status = vec![Status::In; n];
        self.min_take = 1;
        for i in 0..n {
            self.chips[i] -= ANTE as i32;
        }
        let mut cards: Vec<Vec<Rank>> = vec![Vec::new(); n];
        let mut tied: Vec<Seat> = self.seats().collect();
        loop {
            let mut round: Vec<(Seat, Rank)> = Vec::new();
            for &seat in &tied {
                match self.supply.draw() {
                    Some(card) => {
                        cards[seat.index()].push(card);
                        round.push((seat, card));
                    }
                    None => break,
                }
            }
            if round.len() < tied.len() {
                break;
            }
            let low = round.iter().map(|&(_, c)| c).min();
            tied = round
                .iter()
                .filter(|&&(_, c)| Some(c) == low)
                .map(|&(s, _)| s)
                .collect();
            if tied.len() <= 1 {
                break;
            }
        }
        let first = tied.first().copied().unwrap_or(Seat(0));
        let mut returned = Vec::new();
        for (i, dealt) in cards.iter().enumerate() {
            for &card in dealt {
                if !is_black(card) {
                    self.whites.push(card);
                } else if self.blacks[i] == 0 {
                    self.blacks[i] = 1;
                } else {
                    returned.push(card);
                }
            }
        }
        if !returned.is_empty() {
            self.supply.discard_unseen(returned.iter().copied());
            self.supply.reshuffle();
        }
        self.events.push(Event::Opened {
            hand: self.hand_no,
            cards,
            first,
            returned: returned.len() as u8,
        });
        self.phase = Phase::Play;
        self.turn = Some(first);
    }

    fn next_alive(&self, seat: Seat) -> Option<Seat> {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| self.status[s.index()] == Status::In)
    }

    /// [R] Survivors take their antes back and split the pot; odd chips
    /// stay for the next hand.
    fn end_hand(&mut self) {
        self.turn = None;
        let survivors = self.alive();
        for &s in &survivors {
            self.chips[s.index()] += ANTE as i32;
        }
        let share = if survivors.is_empty() {
            0
        } else {
            self.pot / survivors.len() as u32
        };
        for &s in &survivors {
            self.chips[s.index()] += share as i32;
            self.pot -= share;
        }
        self.events.push(Event::PotWon {
            seats: survivors,
            share,
            left: self.pot,
        });
        if self.hand_no >= self.hands_total {
            self.phase = Phase::Over;
        } else {
            self.start_hand();
        }
    }

    /// [R] Cards come one at a time: white stones to the middle, black
    /// ones kept. A second black stone loses at once.
    fn take(&mut self, seat: Seat, count: u8) {
        let i = seat.index();
        let mut drawn = Vec::new();
        let mut lost = false;
        for _ in 0..count {
            let Some(card) = self.supply.draw() else {
                break;
            };
            drawn.push(card);
            if is_black(card) {
                self.blacks[i] += 1;
                if self.blacks[i] >= 2 {
                    lost = true;
                    break;
                }
            } else {
                self.whites.push(card);
            }
        }
        self.events.push(Event::Took {
            seat,
            count,
            cards: drawn,
        });
        self.min_take = count;
        if lost {
            // [R] The ante plus one chip per white stone dealt.
            let extra = self.whites.len() as u32;
            self.chips[i] -= extra as i32;
            self.pot += ANTE + extra;
            self.status[i] = Status::Lost;
            self.events.push(Event::Lost {
                seat,
                paid: ANTE + extra,
            });
            self.end_hand();
            return;
        }
        self.pass(seat);
    }

    fn fold(&mut self, seat: Seat) {
        let cost = self.fold_cost();
        self.chips[seat.index()] -= cost as i32;
        self.pot += ANTE + cost;
        self.status[seat.index()] = Status::Folded;
        self.events.push(Event::Folded {
            seat,
            paid: ANTE + cost,
        });
        self.pass(seat);
    }

    /// [R] One player left wins; otherwise the next one plays.
    fn pass(&mut self, seat: Seat) {
        if self.alive().len() <= 1 {
            self.end_hand();
            return;
        }
        self.turn = self.next_alive(seat);
    }
}

impl Game for Blackstone {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-blackstone",
            name: "Blackstone",
            min_players: 2,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let hands_total = config
            .hands
            .filter(|&h| h > 0)
            .map_or(DEFAULT_HANDS, |h| h.min(MAX_HANDS));
        let n = usize::from(players);
        let mut s = State {
            players,
            hands_total,
            hand_no: 0,
            seed,
            supply: Supply::new(Vec::new(), 0, seed),
            whites: Vec::new(),
            blacks: vec![0; n],
            status: vec![Status::In; n],
            min_take: 1,
            turn: None,
            phase: Phase::Play,
            pot: 0,
            chips: vec![0; n],
            events: Vec::new(),
        };
        s.start_hand();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    /// [R] At least as many cards as the previous player (one for the
    /// first); ❓2 at most what the deck holds.
    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Play || s.turn != Some(seat) {
            return Vec::new();
        }
        let max = s.supply.deck_len().min(usize::from(u8::MAX)) as u8;
        (s.min_take..=max)
            .map(|count| Action::Take { count })
            .chain(std::iter::once(Action::Fold))
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
            return Err(Illegal::new(format!(
                "Prends au moins {} carte(s).",
                s.min_take
            )));
        }
        s.events.clear();
        match *action {
            Action::Take { count } => s.take(seat, count),
            Action::Fold => s.fold(seat),
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
            .chips
            .iter()
            .map(|&mine| s.chips.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
