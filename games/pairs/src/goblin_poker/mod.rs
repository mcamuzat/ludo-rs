//! Goblin Poker (James Ernest): pitch a card into a pool that is shuffled
//! and dealt back, and do not catch a pair. Rules and their sources: RULES.md.

mod determinize;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, lowest, Rank, Supply};

pub struct GoblinPoker;

/// [R] Chips each player pays into the pot at the start of a hand.
pub const ANTE: u32 = 1;
/// [R] Cards dealt to each player.
pub const HAND: usize = 6;
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
    /// Setup: the hand card that starts one's stack.
    Place { card: Rank },
    /// Throw a hand card face down into the pool.
    Pitch { card: Rank },
    /// Pay the lowest card in play and leave the hand.
    Fold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone chooses a first card, at the same time.
    Setup,
    Play,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Setup is over: `seat` had chosen `card`.
    Placed {
        seat: Seat,
        card: Rank,
    },
    /// A card drawn to break a tie for the lowest stack. `kept`: it joined
    /// the stack; otherwise it made a pair and was discarded.
    TieBreak {
        seat: Seat,
        card: Rank,
        kept: bool,
    },
    /// `seat` has the lowest stack and acts first.
    Starts {
        seat: Seat,
    },
    Pitched {
        seat: Seat,
    },
    Folded {
        seat: Seat,
        cost: u32,
    },
    /// A pitched card dealt to `seat`. `paired`: it pays the rank and is out.
    Caught {
        seat: Seat,
        card: Rank,
        paired: bool,
    },
    /// Each of `winners` takes `share` chips from the pot.
    HandEnded {
        winners: Vec<Seat>,
        share: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub hands_total: u8,
    /// The current hand, from 1.
    pub hand_no: u8,
    /// The current round of the hand, from 1; 0 during setup.
    pub round: u8,
    pub supply: Supply,
    /// For each hand's deal and each round's shuffle. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// Face-up cards of each seat. Never two of a rank.
    pub stacks: Vec<Vec<Rank>>,
    /// Setup: the card each seat has chosen, still in its hand.
    pub placed: Vec<Option<Rank>>,
    /// The card each seat has pitched this round, face down.
    pub pitched: Vec<Option<Rank>>,
    /// Folded or paired in the current hand.
    pub out: Vec<bool>,
    pub phase: Phase,
    /// Seat to move during play.
    pub turn: Option<Seat>,
    /// May start a hand with the odd chips of a split pot.
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

const POOL_STREAM: u64 = 0x2545_F491_4F6C_DD1D;

/// [R] Orders stacks from the lowest: the lowest card, then the second
/// lowest, and so on; an extra card is lower than no card.
fn stack_key(stack: &[Rank]) -> Vec<u8> {
    let mut key: Vec<u8> = stack.iter().map(|c| c.value()).collect();
    key.sort_unstable();
    // Above every rank: where a shorter stack ends, it is the higher one.
    key.push(Rank::MAX + 1);
    key
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    /// Seats still in the hand, in seat order.
    pub fn active(&self) -> Vec<Seat> {
        self.seats().filter(|s| !self.out[s.index()]).collect()
    }

    /// Seats after `seat` around the table, `seat` itself last.
    fn around(&self, seat: Seat) -> impl Iterator<Item = Seat> + '_ {
        (1..=self.players).map(move |k| Seat((seat.0 + k) % self.players))
    }

    /// [R] What folding costs: the lowest face-up card of the seats still
    /// in (❓2).
    pub fn fold_cost(&self) -> Option<u32> {
        self.active()
            .iter()
            .filter_map(|s| lowest(&self.stacks[s.index()]))
            .min()
            .map(|card| u32::from(card.value()))
    }

    /// Per rank, the cards `observer` cannot place: not in a stack, the
    /// seen discards, its own hand, nor the card it has pitched.
    pub fn unlocated_for(&self, observer: Option<Seat>) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let own = observer.into_iter().flat_map(|o| {
            self.hands[o.index()]
                .iter()
                .chain(self.pitched[o.index()].iter())
        });
        for card in self
            .stacks
            .iter()
            .flatten()
            .chain(self.supply.seen())
            .chain(own)
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    fn pay(&mut self, seat: Seat, chips: u32) {
        self.chips[seat.index()] -= chips as i32;
        self.pot += chips;
    }

    /// [R] Everyone antes and gets six cards. Leaves `events` alone.
    fn start_hand(&mut self) {
        let n = usize::from(self.players);
        self.hand_no += 1;
        self.round = 0;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.hand_no));
        self.hands = vec![Vec::new(); n];
        for _ in 0..HAND {
            for i in 0..n {
                if let Some(card) = self.supply.draw() {
                    self.hands[i].push(card);
                }
            }
        }
        self.stacks = vec![Vec::new(); n];
        self.placed = vec![None; n];
        self.pitched = vec![None; n];
        self.out = vec![false; n];
        self.phase = Phase::Setup;
        self.turn = None;
        for seat in self.seats().collect::<Vec<_>>() {
            self.pay(seat, ANTE);
        }
    }

    /// [R] The lowest stack among the seats still in. A perfect tie is
    /// broken with cards from the deck, which join the stacks; one that
    /// would make a pair is discarded and replaced.
    fn lowest_stack(&mut self) -> Option<Seat> {
        loop {
            let active = self.active();
            let low = active
                .iter()
                .map(|s| stack_key(&self.stacks[s.index()]))
                .min()?;
            let tied: Vec<Seat> = active
                .into_iter()
                .filter(|s| stack_key(&self.stacks[s.index()]) == low)
                .collect();
            // ❓5: the deck is never rebuilt for a tie; once it is empty
            // the first tied seat is the lowest.
            if tied.len() == 1 || self.supply.deck_len() == 0 {
                return tied.first().copied();
            }
            for seat in tied {
                while self.supply.deck_len() > 0 {
                    let Some(card) = self.supply.draw() else {
                        break;
                    };
                    let kept = !has_rank(&self.stacks[seat.index()], card);
                    self.events.push(Event::TieBreak { seat, card, kept });
                    if kept {
                        self.stacks[seat.index()].push(card);
                        break;
                    }
                    self.supply.discard([card]);
                }
            }
        }
    }

    /// [R] The chosen cards start the stacks.
    fn reveal(&mut self) {
        self.events.clear();
        for seat in self.seats().collect::<Vec<_>>() {
            let i = seat.index();
            let Some(card) = self.placed[i].take() else {
                continue;
            };
            let Some(at) = self.hands[i].iter().position(|&c| c == card) else {
                continue;
            };
            self.hands[i].remove(at);
            self.stacks[i].push(card);
            self.events.push(Event::Placed { seat, card });
        }
        self.phase = Phase::Play;
        self.begin_round();
    }

    /// [R] Each round starts with the lowest stack.
    fn begin_round(&mut self) {
        self.round += 1;
        self.turn = self.lowest_stack();
        if let Some(seat) = self.turn {
            self.events.push(Event::Starts { seat });
        }
    }

    /// ❓6: the stack goes to the discards face up, the hand face down.
    fn leave(&mut self, seat: Seat) {
        let i = seat.index();
        self.out[i] = true;
        let stack = std::mem::take(&mut self.stacks[i]);
        self.supply.discard(stack);
        let hand = std::mem::take(&mut self.hands[i]);
        self.supply.discard_unseen(hand);
    }

    /// After `seat` acted: the hand ends, the pool is dealt, or the turn
    /// passes left.
    fn settle(&mut self, seat: Seat) {
        let active = self.active();
        if let [last] = active.as_slice() {
            // [R] The last player in collects the pot.
            return self.end_hand(vec![*last]);
        }
        let next = self
            .around(seat)
            .find(|s| !self.out[s.index()] && self.pitched[s.index()].is_none());
        match next {
            Some(next) => self.turn = Some(next),
            None => self.deal_pool(),
        }
    }

    /// [R] The pitched cards are shuffled and dealt face up from the lowest
    /// stack; a pair costs its rank and puts the player out, one at a time.
    fn deal_pool(&mut self) {
        let mut pool: Vec<Rank> = self.pitched.iter_mut().filter_map(Option::take).collect();
        let stream =
            mix(self.seed, self.hand_no) ^ POOL_STREAM.wrapping_mul(u64::from(self.round) + 1);
        pool.shuffle(&mut ChaCha8Rng::seed_from_u64(stream));
        let Some(first) = self.lowest_stack() else {
            return;
        };
        let order: Vec<Seat> = std::iter::once(first)
            .chain(self.around(first))
            .take(usize::from(self.players))
            .filter(|s| !self.out[s.index()])
            .collect();
        for seat in order {
            // [R] The last player in wins, even about to catch a pair.
            if self.active().len() == 1 {
                break;
            }
            let Some(card) = pool.pop() else {
                break;
            };
            let paired = has_rank(&self.stacks[seat.index()], card);
            self.events.push(Event::Caught { seat, card, paired });
            if paired {
                self.pay(seat, u32::from(card.value()));
                self.supply.discard([card]);
                self.leave(seat);
            } else {
                self.stacks[seat.index()].push(card);
            }
        }
        self.supply.discard_unseen(pool);
        let active = self.active();
        if active.len() == 1 || active.iter().all(|s| self.hands[s.index()].is_empty()) {
            // [R] One player left, or the fifth round is over.
            self.end_hand(active);
        } else {
            self.begin_round();
        }
    }

    /// [R] The winners share the pot; odd chips stay for the next hand.
    fn end_hand(&mut self, winners: Vec<Seat>) {
        let pool: Vec<Rank> = self.pitched.iter_mut().filter_map(Option::take).collect();
        self.supply.discard_unseen(pool);
        let share = self.pot / (winners.len().max(1) as u32);
        for seat in &winners {
            self.chips[seat.index()] += share as i32;
            self.pot -= share;
        }
        self.events.push(Event::HandEnded { winners, share });
        if self.hand_no >= self.hands_total {
            self.phase = Phase::Over;
            self.turn = None;
        } else {
            self.start_hand();
        }
    }

    /// What `seat` may do on its turn.
    fn play_actions(&self, seat: Seat) -> Vec<Action> {
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        let mut actions: Vec<Action> = cards
            .into_iter()
            .map(|card| Action::Pitch { card })
            .collect();
        actions.push(Action::Fold);
        actions
    }

    fn play(&mut self, seat: Seat, action: &Action) {
        self.events.clear();
        let i = seat.index();
        match *action {
            // Not a play action: `apply` routes it elsewhere.
            Action::Place { .. } => return,
            Action::Pitch { card } => {
                if let Some(at) = self.hands[i].iter().position(|&c| c == card) {
                    self.hands[i].remove(at);
                    self.pitched[i] = Some(card);
                }
                self.events.push(Event::Pitched { seat });
            }
            Action::Fold => {
                let cost = self.fold_cost().unwrap_or(0);
                self.pay(seat, cost);
                self.leave(seat);
                self.events.push(Event::Folded { seat, cost });
            }
        }
        self.settle(seat);
    }
}

impl Game for GoblinPoker {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-goblin-poker",
            name: "Goblin Poker",
            min_players: 3,
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
        let mut s = State {
            players,
            hands_total,
            hand_no: 0,
            round: 0,
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: Vec::new(),
            stacks: Vec::new(),
            placed: Vec::new(),
            pitched: Vec::new(),
            out: Vec::new(),
            phase: Phase::Setup,
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
            Phase::Setup => Pending(
                s.seats()
                    .filter(|x| s.placed[x.index()].is_none())
                    .collect(),
            ),
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    /// During setup nobody learns who has already chosen: a seat only sees
    /// itself pending.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        match (s.phase, viewer) {
            (Phase::Setup, Viewer::Seat(seat)) => match s.placed.get(seat.index()) {
                Some(None) => Pending::one(seat),
                _ => Pending::none(),
            },
            (Phase::Setup, Viewer::Spectator) => Pending::none(),
            _ => Self::pending(s),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat.index() >= usize::from(s.players) {
            return Vec::new();
        }
        match s.phase {
            Phase::Setup if s.placed[seat.index()].is_none() => {
                let mut cards = s.hands[seat.index()].clone();
                cards.sort_unstable();
                cards.dedup();
                cards
                    .into_iter()
                    .map(|card| Action::Place { card })
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
                Phase::Setup => "Tu as déjà posé ta carte.",
                _ => "Ce n'est pas ton tour.",
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Place { card } => {
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
