//! Starboard (James Ernest): everyone plays the same hand in the middle.
//! Pay to add a card to it, or lock, claiming it cannot take another card
//! without pairing. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply};

pub struct Starboard;

/// [R] Each player antes 1 chip.
pub const ANTE: u32 = 1;
/// [R] A hit costs 1 chip.
pub const HIT_COST: u32 = 1;
pub const MAX_ROUNDS: u8 = 30;

/// ❓1: the button goes twice around the table.
pub fn default_rounds(players: u8) -> u8 {
    2 * players
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds in a game, capped at `MAX_ROUNDS`. `None` (or 0): two per
    /// player.
    pub rounds: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Pay a chip and deal a card to the middle.
    Hit,
    /// Claim the middle cannot take another card without pairing.
    Lock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Still to decide.
    In,
    Locked,
    /// Drew a pair.
    Busted,
    /// Locked, then knocked out by a clean draw.
    Knocked,
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
    /// A new round: everyone anted; the two first cards of the middle.
    RoundStarted {
        round: u8,
        button: Seat,
        center: Vec<Rank>,
    },
    /// A clean draw; `knocked`: the locked players it knocked out.
    Hit {
        seat: Seat,
        card: Rank,
        knocked: Vec<Seat>,
    },
    Busted {
        seat: Seat,
        card: Rank,
    },
    /// `auto`: the last player left locks without choosing.
    Locked {
        seat: Seat,
        auto: bool,
    },
    /// The survivors split the pot, `share` each; `left` chips stay for
    /// the next pot.
    RoundEnded {
        winners: Vec<Seat>,
        share: u32,
        left: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub rounds_total: u8,
    /// The current round, from 1.
    pub round_no: u8,
    /// For each round's shuffle. Never in a view.
    pub seed: u64,
    pub supply: Supply,
    /// The hand everyone plays, in the middle.
    pub center: Vec<Rank>,
    pub status: Vec<Status>,
    pub button: Seat,
    pub turn: Option<Seat>,
    pub phase: Phase,
    pub pot: u32,
    /// Each seat's balance since the start; with the pot, they sum to zero.
    pub chips: Vec<i32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    fn pay(&mut self, seat: Seat, chips: u32) {
        self.chips[seat.index()] -= chips as i32;
        self.pot += chips;
    }

    fn with(&self, status: Status) -> Vec<Seat> {
        self.seats()
            .filter(|s| self.status[s.index()] == status)
            .collect()
    }

    /// [R] Everyone antes; two cards of distinct ranks go to the middle (a
    /// pair: one is discarded and replaced). The button acts first.
    fn start_round(&mut self) {
        self.round_no += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.round_no));
        self.center.clear();
        while self.center.len() < 2 {
            let Some(card) = self.supply.draw() else {
                break;
            };
            if has_rank(&self.center, card) {
                self.supply.discard([card]);
            } else {
                self.center.push(card);
            }
        }
        self.status = vec![Status::In; usize::from(self.players)];
        for seat in self.seats().collect::<Vec<_>>() {
            self.pay(seat, ANTE);
        }
        self.events.push(Event::RoundStarted {
            round: self.round_no,
            button: self.button,
            center: self.center.clone(),
        });
        self.turn = Some(self.button);
    }

    /// The next undecided seat to the left of `seat` (itself last); the
    /// last one left locks by itself. With nobody undecided, the round ends.
    fn advance(&mut self, seat: Seat) {
        let alive = self.with(Status::In).len() + self.with(Status::Locked).len();
        if let [last] = self.with(Status::In)[..] {
            if alive == 1 {
                // [R] The only player left automatically locks.
                self.status[last.index()] = Status::Locked;
                self.events.push(Event::Locked {
                    seat: last,
                    auto: true,
                });
            }
        }
        let next = (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| self.status[s.index()] == Status::In);
        match next {
            Some(next) => self.turn = Some(next),
            None => self.end_round(),
        }
    }

    /// [R] The surviving (locked) players split the pot; odd chips stay
    /// for the next one. ❓2: with no survivor, the whole pot stays.
    fn end_round(&mut self) {
        self.turn = None;
        let winners = self.with(Status::Locked);
        let share = if winners.is_empty() {
            0
        } else {
            self.pot / winners.len() as u32
        };
        for &w in &winners {
            self.chips[w.index()] += share as i32;
            self.pot -= share;
        }
        self.events.push(Event::RoundEnded {
            winners,
            share,
            left: self.pot,
        });
        if self.round_no >= self.rounds_total {
            self.phase = Phase::Over;
            return;
        }
        // [R] The button moves one seat to the left.
        self.button = Seat((self.button.0 + 1) % self.players);
        self.start_round();
    }

    /// [R] Pay a chip and deal a card to the middle. A pair busts the
    /// hitter (the card is discarded); a clean draw knocks out every
    /// locked player.
    fn hit(&mut self, seat: Seat) {
        self.pay(seat, HIT_COST);
        let Some(card) = self.supply.draw() else {
            return;
        };
        if has_rank(&self.center, card) {
            self.supply.discard([card]);
            self.status[seat.index()] = Status::Busted;
            self.events.push(Event::Busted { seat, card });
        } else {
            self.center.push(card);
            let knocked = self.with(Status::Locked);
            for s in &knocked {
                self.status[s.index()] = Status::Knocked;
            }
            self.events.push(Event::Hit {
                seat,
                card,
                knocked,
            });
        }
        self.advance(seat);
    }
}

impl Game for Starboard {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-starboard",
            name: "Starboard",
            min_players: 2,
            max_players: 7,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let rounds_total = config
            .rounds
            .filter(|&r| r > 0)
            .map_or(default_rounds(players), |r| r.min(MAX_ROUNDS));
        // [R] The button starts with a random player.
        let button = Seat((ChaCha8Rng::seed_from_u64(seed).next_u32() % u32::from(players)) as u8);
        let mut s = State {
            players,
            rounds_total,
            round_no: 0,
            seed,
            supply: Supply::new(Vec::new(), 0, seed),
            center: Vec::new(),
            status: vec![Status::In; usize::from(players)],
            button,
            turn: None,
            phase: Phase::Play,
            pot: 0,
            chips: vec![0; usize::from(players)],
            events: Vec::new(),
        };
        s.start_round();
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
            return Err(Illegal::new("Il n'y a plus de carte à tirer."));
        }
        s.events.clear();
        match action {
            Action::Hit => s.hit(seat),
            Action::Lock => {
                s.status[seat.index()] = Status::Locked;
                s.events.push(Event::Locked { seat, auto: false });
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
            .chips
            .iter()
            .map(|&mine| s.chips.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
