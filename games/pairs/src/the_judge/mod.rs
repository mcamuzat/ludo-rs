//! The Judge (James Ernest): build a recipe of face-up cards without a
//! pair, then hope the Judge dislikes none of them. Rules and their
//! sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, Rank, Supply};

pub struct TheJudge;

/// [R] Cards dealt to each player.
pub const HAND_SIZE: usize = 7;
/// [R] Cards in the Judge's face-down stack.
pub const JUDGE_SIZE: usize = 8;
/// ❓5: a game stops after this many rounds, at the current ranking.
pub const MAX_ROUNDS: u8 = 40;

/// [R] The score that ends the game: 50 at 2–3 players, 40 at 4, 30 at 5–6.
pub fn target(players: u8) -> u16 {
    match players {
        0..=3 => 50,
        4 => 40,
        _ => 30,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Add a hand card to one's recipe, face up.
    Add { card: Rank },
    /// Stop cooking for this round.
    Pass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Players add cards to their recipes in turn, or pass.
    Cook,
    Over,
}

/// One card of the Judge, revealed, and the recipes it eliminated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reveal {
    pub card: Rank,
    pub eliminated: Vec<Seat>,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Dealt {
        round: u8,
        leader: Seat,
    },
    Added {
        seat: Seat,
        card: Rank,
    },
    /// Also when no card of the hand could be added (❓3): shown as any
    /// pass, which tells nothing about the hand.
    Passed {
        seat: Seat,
    },
    /// The contest: the Judge's cards revealed in order, the recipes that
    /// survived, the points each seat scored, and the totals.
    Contest {
        reveals: Vec<Reveal>,
        survivors: Vec<Seat>,
        gained: Vec<u16>,
        scores: Vec<u16>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    /// The current round, from 1.
    pub round_no: u8,
    /// Its draw pile holds the cards not dealt: nobody sees them.
    pub supply: Supply,
    /// For each round's deal. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// Face-up cards; never a pair.
    pub recipes: Vec<Vec<Rank>>,
    /// Seats that have finished cooking this round.
    pub passed: Vec<bool>,
    /// Face down, revealed from the first.
    pub judge: Vec<Rank>,
    pub scores: Vec<u16>,
    /// Who led this round.
    pub leader: Seat,
    pub phase: Phase,
    pub turn: Seat,
    /// Since the last action of a player.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

/// [R] A recipe scores its face value.
pub fn recipe_points(cards: &[Rank]) -> u16 {
    cards.iter().map(|c| u16::from(c.value())).sum()
}

impl State {
    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// [R] The hand cards `seat` may add: none that would make a pair.
    pub fn addable(&self, seat: Seat) -> Vec<Rank> {
        let recipe = &self.recipes[seat.index()];
        let mut cards: Vec<Rank> = self.hands[seat.index()]
            .iter()
            .copied()
            .filter(|c| !recipe.contains(c))
            .collect();
        cards.sort_unstable();
        cards.dedup();
        cards
    }

    /// [R] Seven cards each, then eight face down for the Judge.
    fn start_round(&mut self, leader: Seat) {
        let n = usize::from(self.players);
        self.round_no += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.round_no));
        self.hands = vec![Vec::new(); n];
        for _ in 0..HAND_SIZE {
            for hand in &mut self.hands {
                if let Some(card) = self.supply.draw() {
                    hand.push(card);
                }
            }
        }
        self.judge = (0..JUDGE_SIZE).filter_map(|_| self.supply.draw()).collect();
        self.recipes = vec![Vec::new(); n];
        self.passed = vec![false; n];
        self.leader = leader;
        self.phase = Phase::Cook;
        self.events.push(Event::Dealt {
            round: self.round_no,
            leader,
        });
        self.settle(leader);
    }

    /// The turn goes to the first seat from `from` still cooking. ❓3: a
    /// seat with no card it may add passes by itself. Everyone passed: the
    /// contest.
    fn settle(&mut self, from: Seat) {
        let mut seat = from;
        for _ in 0..self.players {
            if !self.passed[seat.index()] {
                if !self.addable(seat).is_empty() {
                    self.turn = seat;
                    return;
                }
                self.passed[seat.index()] = true;
                self.events.push(Event::Passed { seat });
            }
            seat = self.next(seat);
        }
        self.contest();
    }

    /// [R] The Judge's cards are revealed one at a time; each eliminates
    /// the recipes holding its rank, until the Judge runs out or one recipe
    /// is left. The survivors score their face value. ❓4: an empty recipe
    /// is not entered.
    fn contest(&mut self) {
        let mut alive: Vec<Seat> = (0..self.players)
            .map(Seat)
            .filter(|s| !self.recipes[s.index()].is_empty())
            .collect();
        let mut reveals = Vec::new();
        let mut revealed = 0;
        while alive.len() > 1 && revealed < self.judge.len() {
            let card = self.judge[revealed];
            revealed += 1;
            let (eliminated, kept): (Vec<Seat>, Vec<Seat>) = alive
                .iter()
                .partition(|s| self.recipes[s.index()].contains(&card));
            alive = kept;
            reveals.push(Reveal { card, eliminated });
        }
        let gained: Vec<u16> = (0..usize::from(self.players))
            .map(|i| {
                if alive.contains(&Seat(i as u8)) {
                    recipe_points(&self.recipes[i])
                } else {
                    0
                }
            })
            .collect();
        for (score, gain) in self.scores.iter_mut().zip(&gained) {
            *score = score.saturating_add(*gain);
        }
        self.events.push(Event::Contest {
            reveals,
            survivors: alive,
            gained: gained.clone(),
            scores: self.scores.clone(),
        });
        // [R] The best score of the round leads the next one (❓1: of tied
        // seats, the first from the previous leader); nobody scored: the
        // same player leads again.
        let best = gained.iter().copied().max().unwrap_or(0);
        let mut leader = self.leader;
        if best > 0 {
            let mut seat = self.leader;
            while gained[seat.index()] != best {
                seat = self.next(seat);
            }
            leader = seat;
        }
        // [R] The game ends once someone reaches the target; ❓2: unless
        // the top is tied, and another round is played.
        let top = self.scores.iter().copied().max().unwrap_or(0);
        let leaders = self.scores.iter().filter(|&&s| s == top).count();
        if (top >= target(self.players) && leaders == 1) || self.round_no >= MAX_ROUNDS {
            self.phase = Phase::Over;
        } else {
            self.start_round(leader);
        }
    }
}

impl Game for TheJudge {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-the-judge",
            name: "The Judge",
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
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: vec![Vec::new(); n],
            recipes: vec![Vec::new(); n],
            passed: vec![false; n],
            judge: Vec::new(),
            scores: vec![0; n],
            leader: Seat(0),
            phase: Phase::Cook,
            turn: Seat(0),
            events: Vec::new(),
        };
        // [R] The dealer leads the first round: a seat drawn from the seed.
        let first = Seat((seed % u64::from(players.max(1))) as u8);
        s.start_round(first);
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Cook => Pending::one(s.turn),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Cook || s.turn != seat {
            return Vec::new();
        }
        s.addable(seat)
            .into_iter()
            .map(|card| Action::Add { card })
            .chain([Action::Pass])
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
            return Err(Illegal::new(match *action {
                Action::Add { card } if s.hands[seat.index()].contains(&card) => {
                    "Cette carte ferait une paire dans ta recette."
                }
                _ => "Ce coup n'est pas permis.",
            }));
        }
        s.events.clear();
        match *action {
            Action::Add { card } => {
                let hand = &mut s.hands[seat.index()];
                if let Some(at) = hand.iter().position(|&c| c == card) {
                    hand.remove(at);
                }
                s.recipes[seat.index()].push(card);
                s.events.push(Event::Added { seat, card });
            }
            Action::Pass => {
                s.passed[seat.index()] = true;
                s.events.push(Event::Passed { seat });
            }
        }
        let next = s.next(seat);
        s.settle(next);
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
