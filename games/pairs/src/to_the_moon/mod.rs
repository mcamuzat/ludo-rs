//! To the Moon! (James Ernest): build a countdown of different ranks and be
//! the last player still in. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply};

pub struct ToTheMoon;

/// [R] A perfect countdown: one card of each rank.
pub const PERFECT: usize = 10;
/// [R]
pub const DEFAULT_ROUNDS: u8 = 6;
pub const MAX_ROUNDS: u8 = 15;

/// [R] Cards dealt to each player: 8 at two, 7 at three, … 4 at six.
pub fn hand_size(players: u8) -> usize {
    usize::from(10u8.saturating_sub(players))
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds in a game, capped at `MAX_ROUNDS`. `None` (or 0): 6.
    pub rounds: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a hand card of a rank not yet in the countdown.
    Build { card: Rank },
    /// Play the top card of the deck.
    Test,
    /// Leave the trick, taking `take` from the countdown (`None` only when
    /// a single card is on the table).
    Withdraw { take: Option<Rank> },
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
    /// A round starts: `dealer` deals.
    Dealt {
        round: u8,
        dealer: Seat,
    },
    /// `seat` starts a countdown with the top card of the deck.
    Led {
        seat: Seat,
        card: Rank,
    },
    Built {
        seat: Seat,
        card: Rank,
    },
    /// `ok`: the card joined the countdown; otherwise it matched and `seat`
    /// is out, with the card in hand.
    Tested {
        seat: Seat,
        card: Rank,
        ok: bool,
    },
    Withdrew {
        seat: Seat,
        took: Option<Rank>,
    },
    /// `seat` scores `cards` points; `perfect`: with all ten ranks.
    TrickWon {
        seat: Seat,
        cards: u32,
        perfect: bool,
    },
    /// The deck is empty: the round is over.
    RoundEnded {
        round: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub rounds_total: u8,
    /// The current round, from 1.
    pub round_no: u8,
    pub dealer: Seat,
    /// Never reshuffled during a round; captured cards go to its discards.
    pub supply: Supply,
    /// For each round's deal. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// The cards of each hand everyone has seen go in: taken from the
    /// countdown, or the card of a failed test.
    pub known: Vec<Vec<Rank>>,
    /// The countdown, in play order. Never two of a rank.
    pub trick: Vec<Rank>,
    /// Who started the current countdown.
    pub leader: Seat,
    /// Out of the current trick.
    pub out: Vec<bool>,
    pub turn: Option<Seat>,
    pub phase: Phase,
    /// Cards captured since the start of the game.
    pub scores: Vec<u32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

fn remove_one(cards: &mut Vec<Rank>, card: Rank) -> bool {
    match cards.iter().position(|&c| c == card) {
        Some(at) => {
            cards.remove(at);
            true
        }
        None => false,
    }
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    /// Seats still in the trick, in seat order.
    pub fn active(&self) -> Vec<Seat> {
        self.seats().filter(|s| !self.out[s.index()]).collect()
    }

    /// The next seat still in after `seat`, clockwise; `seat` itself last.
    fn next_in(&self, seat: Seat) -> Option<Seat> {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| !self.out[s.index()])
    }

    /// The top card, never rebuilt from the discards (❓4).
    fn draw(&mut self) -> Option<Rank> {
        if self.supply.deck_len() == 0 {
            return None;
        }
        self.supply.draw()
    }

    /// Per rank, the cards `observer` cannot place: not in the countdown,
    /// the captured cards, a known card of a hand, nor its own hand.
    pub fn unlocated_for(&self, observer: Option<Seat>) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let own = observer.into_iter().flat_map(|o| &self.hands[o.index()]);
        let known = self
            .known
            .iter()
            .enumerate()
            .filter(|(i, _)| observer.is_none_or(|o| o.index() != *i))
            .flat_map(|(_, cards)| cards);
        for card in self
            .trick
            .iter()
            .chain(self.supply.seen())
            .chain(own)
            .chain(known)
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// [R] Shuffle and deal; the dealer starts the first countdown.
    /// ❓1: the deal passes left each round. Leaves `events` alone.
    fn start_round(&mut self) {
        let n = usize::from(self.players);
        self.round_no += 1;
        self.dealer = Seat((self.round_no - 1) % self.players);
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.round_no));
        self.hands = vec![Vec::new(); n];
        self.known = vec![Vec::new(); n];
        for _ in 0..hand_size(self.players) {
            for k in 1..=n {
                let i = (self.dealer.index() + k) % n;
                if let Some(card) = self.draw() {
                    self.hands[i].push(card);
                }
            }
        }
        self.events.push(Event::Dealt {
            round: self.round_no,
            dealer: self.dealer,
        });
        self.start_trick(self.dealer);
    }

    /// [R] Everyone comes back in and `leader` plays the top card of the
    /// deck; an empty deck ends the round. ❓2: the seat after the leader
    /// acts first.
    fn start_trick(&mut self, leader: Seat) {
        self.out = vec![false; usize::from(self.players)];
        self.leader = leader;
        let Some(card) = self.draw() else {
            return self.end_round();
        };
        self.trick = vec![card];
        self.events.push(Event::Led { seat: leader, card });
        self.turn = self.next_in(leader);
    }

    /// ❓6: the cards left in hand score nothing.
    fn end_round(&mut self) {
        for hand in &mut self.hands {
            self.supply.discard_unseen(std::mem::take(hand));
        }
        self.known.iter_mut().for_each(Vec::clear);
        self.trick.clear();
        self.events.push(Event::RoundEnded {
            round: self.round_no,
        });
        if self.round_no >= self.rounds_total {
            self.phase = Phase::Over;
            self.turn = None;
        } else {
            self.start_round();
        }
    }

    /// [R] `seat` scores one point per card and leads the next countdown.
    fn take_trick(&mut self, seat: Seat) {
        let cards = std::mem::take(&mut self.trick);
        let perfect = cards.len() == PERFECT;
        self.scores[seat.index()] += cards.len() as u32;
        self.events.push(Event::TrickWon {
            seat,
            cards: cards.len() as u32,
            perfect,
        });
        self.supply.discard(cards);
        self.start_trick(seat);
    }

    /// After `seat` acted: a perfect countdown or a last player takes the
    /// trick; otherwise the turn passes clockwise.
    fn settle(&mut self, seat: Seat) {
        // [R] A perfect countdown wins at once; ❓3 also by a test.
        if self.trick.len() == PERFECT {
            return self.take_trick(seat);
        }
        let active = self.active();
        if let [last] = active.as_slice() {
            return self.take_trick(*last);
        }
        self.turn = self.next_in(seat);
    }

    fn play_actions(&self, seat: Seat) -> Vec<Action> {
        let mut cards: Vec<Rank> = self.hands[seat.index()]
            .iter()
            .copied()
            .filter(|&c| !has_rank(&self.trick, c))
            .collect();
        cards.sort_unstable();
        cards.dedup();
        let mut actions: Vec<Action> = cards
            .into_iter()
            .map(|card| Action::Build { card })
            .collect();
        if self.supply.deck_len() > 0 {
            actions.push(Action::Test);
        }
        // [R] Taking a card may not leave the table empty.
        if self.trick.len() >= 2 {
            let mut takes = self.trick.clone();
            takes.sort_unstable();
            actions.extend(
                takes
                    .into_iter()
                    .map(|card| Action::Withdraw { take: Some(card) }),
            );
        } else {
            actions.push(Action::Withdraw { take: None });
        }
        actions
    }

    fn play(&mut self, seat: Seat, action: &Action) {
        self.events.clear();
        let i = seat.index();
        match *action {
            Action::Build { card } => {
                remove_one(&mut self.hands[i], card);
                remove_one(&mut self.known[i], card);
                self.trick.push(card);
                self.events.push(Event::Built { seat, card });
            }
            Action::Test => {
                let Some(card) = self.draw() else {
                    return;
                };
                let ok = !has_rank(&self.trick, card);
                if ok {
                    self.trick.push(card);
                } else {
                    // [R] Knocked out, with the card in hand.
                    self.out[i] = true;
                    self.hands[i].push(card);
                    self.known[i].push(card);
                }
                self.events.push(Event::Tested { seat, card, ok });
            }
            Action::Withdraw { take } => {
                self.out[i] = true;
                if let Some(card) = take {
                    remove_one(&mut self.trick, card);
                    self.hands[i].push(card);
                    self.known[i].push(card);
                }
                self.events.push(Event::Withdrew { seat, took: take });
            }
        }
        self.settle(seat);
    }
}

impl Game for ToTheMoon {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-to-the-moon",
            name: "To the Moon!",
            min_players: 2,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let rounds_total = config
            .rounds
            .filter(|&r| r > 0)
            .map_or(DEFAULT_ROUNDS, |r| r.min(MAX_ROUNDS));
        let mut s = State {
            players,
            rounds_total,
            round_no: 0,
            dealer: Seat(0),
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: Vec::new(),
            known: Vec::new(),
            trick: Vec::new(),
            leader: Seat(0),
            out: Vec::new(),
            turn: None,
            phase: Phase::Play,
            scores: vec![0; usize::from(players)],
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
        if s.phase == Phase::Play && s.turn == Some(seat) {
            s.play_actions(seat)
        } else {
            Vec::new()
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        s.play(seat, action);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        // [R] The highest total wins; ❓5: equal totals stay tied.
        let ranks = s
            .scores
            .iter()
            .map(|&mine| s.scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
