//! Troll Guts (James Ernest): a chaotic Goblin Poker with no folding.
//! Draw, steal or buy a face-up card until a pair knocks you out; the
//! survivor takes the pot. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, open_round, Opening, Rank, Supply};

pub struct TrollGuts;

/// [R] Each player antes 1 chip.
pub const ANTE: u32 = 1;
/// [R] Buying one's own top card costs 1 chip.
pub const BUY_COST: u32 = 1;
/// [R] Five face-down cards each.
pub const STACK_SIZE: usize = 5;
/// ❓1
pub const DEFAULT_HANDS: u8 = 5;
pub const MAX_HANDS: u8 = 15;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Hands in a game, capped at `MAX_HANDS`. `None` (or 0): 5.
    pub hands: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// The order of one's face-down stack, top card first (in secret).
    Arrange { order: Vec<Rank> },
    /// Take a card from the deck.
    Draw,
    /// Take the top card of another player's face-down stack.
    Steal { from: Seat },
    /// Pay a chip and take the top card of one's own stack.
    Buy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone orders its face-down stack, at the same time.
    Arrange,
    Play,
    Over,
}

/// Where a new face-up card came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Source {
    Deck,
    Stack { seat: Seat },
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new hand: everyone anted; `first` plays first.
    HandStarted {
        hand: u8,
        first: Seat,
    },
    /// An opening face-up card (ties for low get more).
    Dealt {
        seat: Seat,
        card: Rank,
    },
    /// An opening tie-break card that would pair: discarded and replaced.
    Replaced {
        seat: Seat,
        card: Rank,
    },
    /// Everyone has ordered its stack.
    Arranged,
    Took {
        seat: Seat,
        card: Rank,
        from: Source,
    },
    /// `seat` paired `card`, paid its value and is out.
    Busted {
        seat: Seat,
        card: Rank,
        paid: u32,
    },
    Reshuffled,
    /// `seats` (the survivors) take the pot, `share` each.
    PotWon {
        seats: Vec<Seat>,
        share: u32,
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
    /// Face-down stacks; the top is the last element.
    pub stacks: Vec<Vec<Rank>>,
    /// Arrange: the order each seat has chosen (top first), not yet set.
    pub placed: Vec<Option<Vec<Rank>>>,
    pub faceup: Vec<Vec<Rank>>,
    pub out: Vec<bool>,
    /// Who plays first once the stacks are ordered.
    pub first: Seat,
    pub turn: Option<Seat>,
    pub phase: Phase,
    pub pot: u32,
    /// Each seat's balance since the start; with the pot, they sum to zero.
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
        self.seats().filter(|s| !self.out[s.index()]).collect()
    }

    fn pay(&mut self, seat: Seat, chips: u32) {
        self.chips[seat.index()] -= chips as i32;
        self.pot += chips;
    }

    fn note(&mut self, log: Vec<Opening>) {
        self.events.extend(log.into_iter().map(|o| match o {
            Opening::Dealt { seat, card } => Event::Dealt { seat, card },
            Opening::Replaced { seat, card } => Event::Replaced { seat, card },
            Opening::Reshuffled => Event::Reshuffled,
        }));
    }

    /// [R] Everyone antes; five cards face down and one face up each; the
    /// low face-up card goes first, ties broken as in Pairs (the extra
    /// cards stay).
    fn start_hand(&mut self) {
        self.hand_no += 1;
        let n = usize::from(self.players);
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.hand_no));
        self.stacks = vec![Vec::new(); n];
        for _ in 0..STACK_SIZE {
            for i in 0..n {
                if let Some(card) = self.supply.draw() {
                    self.stacks[i].push(card);
                }
            }
        }
        self.faceup = vec![Vec::new(); n];
        self.out = vec![false; n];
        self.placed = vec![None; n];
        for seat in self.seats().collect::<Vec<_>>() {
            self.pay(seat, ANTE);
        }
        let seats: Vec<Seat> = self.seats().collect();
        let mut log = Vec::new();
        self.first = open_round(&mut self.supply, &mut self.faceup, &seats, &mut log);
        self.events.push(Event::HandStarted {
            hand: self.hand_no,
            first: self.first,
        });
        self.note(log);
        self.phase = Phase::Arrange;
        self.turn = None;
    }

    /// [R] Each stack takes its chosen order.
    fn arrange_all(&mut self) {
        let placed = std::mem::take(&mut self.placed);
        for (i, order) in placed.into_iter().enumerate() {
            if let Some(mut order) = order {
                order.reverse(); // top last
                self.stacks[i] = order;
            }
        }
        self.placed = vec![None; usize::from(self.players)];
        self.events.push(Event::Arranged);
        self.phase = Phase::Play;
        self.begin_turn(self.first);
    }

    /// What `seat` may take now.
    fn take_actions(&self, seat: Seat) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.supply.can_draw() {
            actions.push(Action::Draw);
        }
        for from in self.alive() {
            if from != seat && !self.stacks[from.index()].is_empty() {
                actions.push(Action::Steal { from });
            }
        }
        if !self.stacks[seat.index()].is_empty() {
            actions.push(Action::Buy);
        }
        actions
    }

    /// `seat` plays, or (❓3) with nothing left to take anywhere, the
    /// survivors share the pot.
    fn begin_turn(&mut self, seat: Seat) {
        if self.take_actions(seat).is_empty() {
            let survivors = self.alive();
            self.end_hand(&survivors);
        } else {
            self.turn = Some(seat);
        }
    }

    fn next_alive(&self, seat: Seat) -> Option<Seat> {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| !self.out[s.index()])
    }

    /// [R] The new card goes face up; a pair costs its value and knocks
    /// the player out. The last survivor takes the pot.
    fn take(&mut self, seat: Seat, card: Rank, from: Source) {
        let i = seat.index();
        if has_rank(&self.faceup[i], card) {
            let paid = u32::from(card.value());
            self.faceup[i].push(card);
            self.pay(seat, paid);
            self.out[i] = true;
            self.events.push(Event::Busted { seat, card, paid });
            // [R] A knocked-out player turns its cards face down.
            let faceup = std::mem::take(&mut self.faceup[i]);
            let stack = std::mem::take(&mut self.stacks[i]);
            self.supply.discard_unseen(faceup.into_iter().chain(stack));
        } else {
            self.faceup[i].push(card);
            self.events.push(Event::Took { seat, card, from });
        }
        let alive = self.alive();
        if let [last] = alive[..] {
            self.end_hand(&[last]);
            return;
        }
        match self.next_alive(seat) {
            Some(next) => self.begin_turn(next),
            None => self.end_hand(&[]),
        }
    }

    fn end_hand(&mut self, winners: &[Seat]) {
        self.turn = None;
        let share = if winners.is_empty() {
            0
        } else {
            self.pot / winners.len() as u32
        };
        for &w in winners {
            self.chips[w.index()] += share as i32;
            self.pot -= share;
        }
        self.events.push(Event::PotWon {
            seats: winners.to_vec(),
            share,
        });
        if self.hand_no >= self.hands_total {
            self.phase = Phase::Over;
        } else {
            self.start_hand();
        }
    }

    /// Every distinct order of `cards`, top first.
    pub fn orders(cards: &[Rank]) -> Vec<Vec<Rank>> {
        fn go(rest: &mut Vec<Rank>, prefix: &mut Vec<Rank>, out: &mut Vec<Vec<Rank>>) {
            if rest.is_empty() {
                out.push(prefix.clone());
                return;
            }
            let mut tried: Vec<Rank> = Vec::new();
            for k in 0..rest.len() {
                let card = rest[k];
                if tried.contains(&card) {
                    continue;
                }
                tried.push(card);
                rest.remove(k);
                prefix.push(card);
                go(rest, prefix, out);
                prefix.pop();
                rest.insert(k, card);
            }
        }
        let mut rest = cards.to_vec();
        rest.sort_unstable();
        let mut out = Vec::new();
        go(&mut rest, &mut Vec::new(), &mut out);
        out
    }
}

impl Game for TrollGuts {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-troll-guts",
            name: "Troll Guts",
            min_players: 3,
            max_players: 6,
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
            stacks: vec![Vec::new(); n],
            placed: vec![None; n],
            faceup: vec![Vec::new(); n],
            out: vec![false; n],
            first: Seat(0),
            turn: None,
            phase: Phase::Arrange,
            pot: 0,
            chips: vec![0; n],
            events: Vec::new(),
        };
        s.start_hand();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Arrange => Pending(
                s.seats()
                    .filter(|x| s.placed[x.index()].is_none())
                    .collect(),
            ),
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    /// While stacks are ordered nobody learns who has already done so.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        match (s.phase, viewer) {
            (Phase::Arrange, Viewer::Seat(seat)) => match s.placed.get(seat.index()) {
                Some(None) => Pending::one(seat),
                _ => Pending::none(),
            },
            (Phase::Arrange, Viewer::Spectator) => Pending::none(),
            _ => Self::pending(s),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if !Self::pending(s).contains(seat) {
            return Vec::new();
        }
        match s.phase {
            Phase::Arrange => State::orders(&s.stacks[seat.index()])
                .into_iter()
                .map(|order| Action::Arrange { order })
                .collect(),
            Phase::Play => s.take_actions(seat),
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if !Self::pending(s).contains(seat) {
            return Err(Illegal::new(match s.phase {
                Phase::Arrange => "Tu as déjà rangé ta pile.",
                _ => "Ce n'est pas ton tour.",
            }));
        }
        let legal = match action {
            // Checking an order needs no list of all 120.
            Action::Arrange { order } => {
                s.phase == Phase::Arrange && {
                    let mut a = order.clone();
                    let mut b = s.stacks[seat.index()].clone();
                    a.sort_unstable();
                    b.sort_unstable();
                    a == b
                }
            }
            _ => s.phase == Phase::Play && s.take_actions(seat).contains(action),
        };
        if !legal {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match action {
            Action::Arrange { order } => {
                s.placed[seat.index()] = Some(order.clone());
                if s.placed.iter().all(Option::is_some) {
                    s.arrange_all();
                }
            }
            Action::Draw => {
                s.events.clear();
                let before = s.supply.shuffles();
                if let Some(card) = s.supply.draw() {
                    if s.supply.shuffles() != before {
                        s.events.push(Event::Reshuffled);
                    }
                    s.take(seat, card, Source::Deck);
                }
            }
            Action::Steal { from } => {
                s.events.clear();
                if let Some(card) = s.stacks[from.index()].pop() {
                    s.take(seat, card, Source::Stack { seat: *from });
                }
            }
            Action::Buy => {
                s.events.clear();
                s.pay(seat, BUY_COST);
                if let Some(card) = s.stacks[seat.index()].pop() {
                    s.take(seat, card, Source::Stack { seat });
                }
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
