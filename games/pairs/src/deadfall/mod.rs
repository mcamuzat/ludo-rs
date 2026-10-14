//! Deadfall (James Ernest): play a card to claim its rank is still alive,
//! or call another player's top card dead. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply};

pub struct Deadfall;

/// [R] Chips each player pays into the pot at the start of a hand.
pub const ANTE: u32 = 1;
/// ❓1
pub const DEFAULT_HANDS: u8 = 5;
pub const MAX_HANDS: u8 = 15;

/// [R] Six cards each; seven at two players, five at seven.
pub fn hand_size(players: u8) -> usize {
    match players {
        2 => 7,
        7 => 5,
        _ => 6,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Hands in a game, capped at `MAX_HANDS`. `None` (or 0): 5.
    pub hands: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Door: the hand card chosen in secret, revealed with the others.
    Door { card: Rank },
    /// Play a hand card on one's stack.
    Play { card: Rank },
    /// Claim that the top card of `seat` is dead; ends the hand.
    Call { seat: Seat },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone chooses a door card, at the same time.
    Door,
    Play,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// The door cards of every seat. `leader`: who plays first; `None` on
    /// a tie, and everyone plays another door card.
    DoorsRevealed {
        cards: Vec<Rank>,
        leader: Option<Seat>,
    },
    Played {
        seat: Seat,
        card: Rank,
    },
    /// `caller` called the `rank` on top of `target`; every hand is shown.
    /// `winner` takes `paid` chips from the loser and the `pot`.
    Showdown {
        caller: Seat,
        target: Seat,
        rank: Rank,
        dead: bool,
        hands: Vec<Vec<Rank>>,
        winner: Seat,
        paid: u32,
        pot: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub hands_total: u8,
    /// The current hand, from 1.
    pub hand_no: u8,
    /// Its draw pile holds the cards not dealt: nobody sees them.
    pub supply: Supply,
    /// For each hand's deal. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// Face-up cards of each seat; the top is the last.
    pub stacks: Vec<Vec<Rank>>,
    /// When the top card of each seat was played (`clock` order).
    pub stamps: Vec<u32>,
    pub clock: u32,
    /// Door: the card each seat has chosen, still in its hand.
    pub placed: Vec<Option<Rank>>,
    pub phase: Phase,
    pub turn: Option<Seat>,
    pub pot: u32,
    /// Each seat's balance since the start of the game; may be negative.
    /// With the pot, they always sum to zero.
    pub chips: Vec<i32>,
    /// Since the previous decision in play, or since the last reveal.
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

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// [R] The top cards `caller` may call: any other seat's, but of
    /// several top cards of one rank only the one played last (❓2: among
    /// the other seats).
    pub fn callable(&self, caller: Seat) -> Vec<Seat> {
        let tops: Vec<(Seat, Rank, u32)> = self
            .seats()
            .filter(|&s| s != caller)
            .filter_map(|s| {
                self.stacks[s.index()]
                    .last()
                    .map(|&r| (s, r, self.stamps[s.index()]))
            })
            .collect();
        tops.iter()
            .filter(|&&(_, rank, stamp)| {
                tops.iter()
                    .all(|&(_, other, at)| other != rank || at <= stamp)
            })
            .map(|&(seat, _, _)| seat)
            .collect()
    }

    fn pay(&mut self, seat: Seat, chips: u32) {
        self.chips[seat.index()] -= chips as i32;
        self.pot += chips;
    }

    /// [R] Everyone antes and is dealt a hand. Leaves `events` alone.
    fn start_hand(&mut self) {
        let n = usize::from(self.players);
        self.hand_no += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.hand_no));
        self.hands = vec![Vec::new(); n];
        for _ in 0..hand_size(self.players) {
            for i in 0..n {
                if let Some(card) = self.supply.draw() {
                    self.hands[i].push(card);
                }
            }
        }
        self.stacks = vec![Vec::new(); n];
        self.stamps = vec![0; n];
        self.clock = 0;
        self.placed = vec![None; n];
        self.phase = Phase::Door;
        self.turn = None;
        for seat in self.seats().collect::<Vec<_>>() {
            self.pay(seat, ANTE);
        }
    }

    /// [R] The door cards are revealed together. The lowest goes first;
    /// on a tie everyone plays another door card, the latest deciding and
    /// the earlier ones breaking ties.
    fn reveal(&mut self) {
        self.events.clear();
        let mut cards = Vec::new();
        for i in 0..usize::from(self.players) {
            if let Some(card) = self.placed[i].take() {
                if let Some(at) = self.hands[i].iter().position(|&c| c == card) {
                    self.hands[i].remove(at);
                }
                self.stacks[i].push(card);
                cards.push(card);
            }
        }
        // Latest door card first, then the earlier ones.
        let key = |s: Seat| -> Vec<Rank> { self.stacks[s.index()].iter().rev().copied().collect() };
        let low = self.seats().map(key).min().unwrap_or_default();
        let tied: Vec<Seat> = self.seats().filter(|&s| key(s) == low).collect();
        // ❓3: when the hands are empty, the first tied seat leads.
        let leader = (tied.len() == 1 || self.hands.iter().all(Vec::is_empty))
            .then(|| tied.first().copied())
            .flatten();
        self.events.push(Event::DoorsRevealed { cards, leader });
        let Some(leader) = leader else {
            return;
        };
        // [R] The door cards count as played clockwise from the leader.
        for k in 0..self.players {
            let seat = Seat((leader.0 + k) % self.players);
            self.clock += 1;
            self.stamps[seat.index()] = self.clock;
        }
        self.phase = Phase::Play;
        self.turn = Some(leader);
    }

    /// [R] Every hand is shown. The called rank is dead when no hand holds
    /// it: the caller wins; otherwise the called player does. The loser
    /// pays the winner the rank, and the winner takes the antes.
    fn showdown(&mut self, caller: Seat, target: Seat) {
        let Some(&rank) = self.stacks[target.index()].last() else {
            return;
        };
        let dead = !self.hands.iter().any(|h| has_rank(h, rank));
        let (winner, loser) = if dead {
            (caller, target)
        } else {
            (target, caller)
        };
        let paid = u32::from(rank.value());
        self.chips[loser.index()] -= paid as i32;
        self.chips[winner.index()] += paid as i32 + self.pot as i32;
        let pot = std::mem::take(&mut self.pot);
        self.events.push(Event::Showdown {
            caller,
            target,
            rank,
            dead,
            hands: self.hands.clone(),
            winner,
            paid,
            pot,
        });
        for i in 0..usize::from(self.players) {
            let stack = std::mem::take(&mut self.stacks[i]);
            self.supply.discard(stack);
            let hand = std::mem::take(&mut self.hands[i]);
            self.supply.discard(hand);
        }
        if self.hand_no >= self.hands_total {
            self.phase = Phase::Over;
            self.turn = None;
        } else {
            self.start_hand();
        }
    }

    /// What `seat` may do on its turn. [R] With an empty hand, only call.
    fn play_actions(&self, seat: Seat) -> Vec<Action> {
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        cards
            .into_iter()
            .map(|card| Action::Play { card })
            .chain(
                self.callable(seat)
                    .into_iter()
                    .map(|seat| Action::Call { seat }),
            )
            .collect()
    }

    fn play(&mut self, seat: Seat, action: &Action) {
        self.events.clear();
        match *action {
            // Not a play action: `apply` routes it elsewhere.
            Action::Door { .. } => {}
            Action::Play { card } => {
                let i = seat.index();
                if let Some(at) = self.hands[i].iter().position(|&c| c == card) {
                    self.hands[i].remove(at);
                }
                self.stacks[i].push(card);
                self.clock += 1;
                self.stamps[i] = self.clock;
                self.events.push(Event::Played { seat, card });
                self.turn = Some(self.next(seat));
            }
            Action::Call { seat: target } => self.showdown(seat, target),
        }
    }
}

impl Game for Deadfall {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-deadfall",
            name: "Deadfall",
            min_players: 2,
            max_players: 7,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let hands_total = config
            .hands
            .filter(|&h| h > 0)
            .map_or(DEFAULT_HANDS, |h| h.min(MAX_HANDS));
        let mut s = State {
            players,
            hands_total,
            hand_no: 0,
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: Vec::new(),
            stacks: Vec::new(),
            stamps: Vec::new(),
            clock: 0,
            placed: Vec::new(),
            phase: Phase::Door,
            turn: None,
            pot: 0,
            chips: vec![0; usize::from(players)],
            events: Vec::new(),
        };
        s.start_hand();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Door => Pending(
                s.seats()
                    .filter(|x| s.placed[x.index()].is_none())
                    .collect(),
            ),
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    /// While door cards are chosen nobody learns who has already chosen:
    /// a seat only sees itself pending.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        match (s.phase, viewer) {
            (Phase::Door, Viewer::Seat(seat)) => match s.placed.get(seat.index()) {
                Some(None) => Pending::one(seat),
                _ => Pending::none(),
            },
            (Phase::Door, Viewer::Spectator) => Pending::none(),
            _ => Self::pending(s),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat.index() >= usize::from(s.players) {
            return Vec::new();
        }
        match s.phase {
            Phase::Door if s.placed[seat.index()].is_none() => {
                let mut cards = s.hands[seat.index()].clone();
                cards.sort_unstable();
                cards.dedup();
                cards
                    .into_iter()
                    .map(|card| Action::Door { card })
                    .collect()
            }
            Phase::Play if s.turn == Some(seat) => s.play_actions(seat),
            _ => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new(match s.phase {
                Phase::Door => "Tu as déjà choisi ta carte de porte.",
                _ => "Ce n'est pas ton tour.",
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Door { card } => {
                s.placed[seat.index()] = Some(card);
                if s.placed.iter().all(Option::is_some) {
                    s.reveal();
                }
            }
            _ => s.play(seat, action),
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
        // ❓1: ranked by chips; equal balances stay tied.
        let ranks = s
            .chips
            .iter()
            .map(|&mine| s.chips.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
