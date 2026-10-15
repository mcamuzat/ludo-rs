//! Octopus (James Ernest): a high-hand game with rounds of card passing,
//! where whoever calls the showdown picks which kind of hand is best.
//! Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{count_by_rank, full_deck, Rank};

pub struct Octopus;

/// [R] Eight cards each.
pub const HAND_SIZE: usize = 8;
/// [R] The usual ante.
pub const ANTE: u32 = 1;
/// [R] The ante on the first hand, and after a hand won by folds.
pub const OPEN_ANTE: u32 = 2;
/// [R] The ante of the loser of the last hand.
pub const PENALTY: u32 = 5;
/// [R] What folding costs.
pub const FOLD_COST: u32 = 1;
/// ❓1
pub const DEFAULT_HANDS: u8 = 5;
pub const MAX_HANDS: u8 = 15;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Hands in a game, capped at `MAX_HANDS`. `None` (or 0): 5.
    pub hands: Option<u8>,
}

/// [R] The three types of hands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    /// No unique card: 99-888-33.
    Set,
    /// No duplicated card: 3-4-5-7-9.
    Clean,
    /// Both: always the worst.
    Dirty,
}

impl Kind {
    pub fn of(hand: &[Rank]) -> Kind {
        let counts = count_by_rank(hand);
        let present = counts.iter().filter(|&&c| c > 0);
        if present.clone().all(|&c| c == 1) {
            Kind::Clean
        } else if present.clone().all(|&c| c >= 2) {
            Kind::Set
        } else {
            Kind::Dirty
        }
    }
}

/// [R] How a hand ranks when `best` is the preferred kind: the preferred
/// kind first, then the other clean or set kind, then dirty; within a
/// kind, the highest card is the worst, then the next, and so on. Lower
/// is better.
pub fn strength(hand: &[Rank], best: Kind) -> (u8, Vec<Rank>) {
    let kind = Kind::of(hand);
    let order = match kind {
        Kind::Dirty => 2,
        k if k == best => 0,
        _ => 1,
    };
    let mut cards = hand.to_vec();
    cards.sort_unstable_by(|a, b| b.cmp(a));
    (order, cards)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Declare, in turn: stay in the hand, or fold for a chip.
    Stay,
    Fold,
    /// The card played this round, chosen in secret.
    Play {
        card: Rank,
    },
    /// The lowest unique card calls: another round of passing…
    CallPass,
    /// … or the showdown, where `best` hands win.
    Show {
        best: Kind,
    },
    /// The card passed to the next player on the left, chosen in secret.
    Pass {
        card: Rank,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// One after another, from the leader.
    Declare,
    /// Everyone still in plays a card, at the same time.
    Play,
    /// The lowest unique card decides.
    Call,
    /// Everyone still in passes a card, at the same time.
    Pass,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new hand: what each seat anted, and who declares first.
    HandStarted {
        hand: u8,
        antes: Vec<u32>,
        leader: Seat,
    },
    Declared {
        seat: Seat,
        fold: bool,
    },
    /// Everyone else folded: `seat` takes the pot.
    FoldedOut {
        seat: Seat,
        pot: u32,
    },
    /// The cards played this round (`None`: folded). `caller`: the lowest
    /// unique card, if any.
    CardsPlayed {
        cards: Vec<Option<Rank>>,
        caller: Option<Seat>,
    },
    /// The lowest unique card chose another round of passing.
    PassCalled {
        seat: Seat,
    },
    /// Everyone still in passed a card to the left.
    Passed,
    /// The hands are shown. `best`: the kind the caller chose (`None`
    /// when the hands ran down to one card). `winners` share `share`
    /// chips each; `losers` pay the penalty, into the next pot or, on the
    /// last hand, to the winners.
    Showdown {
        caller: Option<Seat>,
        best: Option<Kind>,
        hands: Vec<Option<Vec<Rank>>>,
        winners: Vec<Seat>,
        losers: Vec<Seat>,
        share: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub hands_total: u8,
    /// The current hand, from 1.
    pub hand_no: u8,
    /// For each hand's deal. Never in a view.
    pub seed: u64,
    /// Cards not dealt: nobody sees them.
    pub undealt: Vec<Rank>,
    pub hands: Vec<Vec<Rank>>,
    pub folded: Vec<bool>,
    /// Who declares first this hand, and starts the next if it wins.
    pub leader: Seat,
    /// Declare: whose turn it is.
    pub turn: Option<Seat>,
    /// Play or Pass: the card each seat has chosen, still in its hand.
    pub placed: Vec<Option<Rank>>,
    /// Since the last pass: the card each seat passed (known to it and to
    /// the player it went to only).
    pub passed: Vec<Option<Rank>>,
    pub received: Vec<Option<Rank>>,
    /// The card each seat passed, while its receiver has not played a
    /// card of that rank since: the passer knows where it is.
    pub tracked: Vec<Option<Rank>>,
    /// The cards played this hand, face up.
    pub played: Vec<Rank>,
    /// Call: the lowest unique card.
    pub caller: Option<Seat>,
    pub phase: Phase,
    pub pot: u32,
    /// Each seat's balance since the start of the game; may be negative.
    /// With the pot, they always sum to zero.
    pub chips: Vec<i32>,
    /// Who pays the penalty ante on the next hand. `None`: nobody lost
    /// (first hand, or a hand won by folds): everyone antes 2.
    pub losers: Option<Vec<Seat>>,
    /// Since the previous decision in turn, or since the last reveal.
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

    pub fn active(&self) -> Vec<Seat> {
        self.seats().filter(|s| !self.folded[s.index()]).collect()
    }

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// The next seat still in, to the left of `seat`.
    pub fn next_active(&self, seat: Seat) -> Seat {
        let mut s = self.next(seat);
        while self.folded[s.index()] && s != seat {
            s = self.next(s);
        }
        s
    }

    fn pay(&mut self, seat: Seat, chips: u32) {
        self.chips[seat.index()] -= chips as i32;
        self.pot += chips;
    }

    fn take_from_hand(&mut self, seat: Seat, card: Rank) {
        let hand = &mut self.hands[seat.index()];
        if let Some(at) = hand.iter().position(|&c| c == card) {
            hand.remove(at);
        }
    }

    fn reset_choices(&mut self) {
        self.placed = vec![None; usize::from(self.players)];
    }

    /// [R] Everyone antes (2 on the first hand or after a hand won by
    /// folds; otherwise 1, and 5 for the loser), and is dealt 8 cards.
    fn start_hand(&mut self) {
        let n = usize::from(self.players);
        self.hand_no += 1;
        let mut deck = full_deck();
        deck.shuffle(&mut ChaCha8Rng::seed_from_u64(mix(self.seed, self.hand_no)));
        self.hands = (0..n)
            .map(|_| deck.split_off(deck.len() - HAND_SIZE))
            .collect();
        self.undealt = deck;
        self.folded = vec![false; n];
        self.reset_choices();
        self.passed = vec![None; n];
        self.received = vec![None; n];
        self.tracked = vec![None; n];
        self.played.clear();
        self.caller = None;
        let antes: Vec<u32> = self
            .seats()
            .map(|s| match &self.losers {
                None => OPEN_ANTE,
                Some(losers) if losers.contains(&s) => PENALTY,
                Some(_) => ANTE,
            })
            .collect();
        for (i, &ante) in antes.iter().enumerate() {
            self.pay(Seat(i as u8), ante);
        }
        self.events.push(Event::HandStarted {
            hand: self.hand_no,
            antes,
            leader: self.leader,
        });
        self.phase = Phase::Declare;
        self.turn = Some(self.leader);
    }

    fn end_hand(&mut self) {
        let hands = std::mem::take(&mut self.hands);
        self.played.extend(hands.into_iter().flatten());
        self.turn = None;
        self.caller = None;
        if self.hand_no >= self.hands_total {
            self.phase = Phase::Over;
        } else {
            self.start_hand();
        }
    }

    /// [R] Declaring goes once around the table from the leader. When
    /// everyone but one has folded, that one takes the pot and leads the
    /// next hand, where everyone antes 2.
    fn declare(&mut self, seat: Seat, fold: bool) {
        self.events.clear();
        if fold {
            self.pay(seat, FOLD_COST);
            self.folded[seat.index()] = true;
        }
        self.events.push(Event::Declared { seat, fold });
        let active = self.active();
        if let [winner] = active[..] {
            let pot = std::mem::take(&mut self.pot);
            self.chips[winner.index()] += pot as i32;
            self.events.push(Event::FoldedOut { seat: winner, pot });
            self.leader = winner;
            self.losers = None;
            self.end_hand();
            return;
        }
        let next = self.next(seat);
        if next == self.leader {
            self.turn = None;
            self.phase = Phase::Play;
        } else {
            self.turn = Some(next);
        }
    }

    /// [R] The cards are revealed together (and discarded, ❓2). The
    /// lowest unique card calls; without one, another round, unless the
    /// hands are down to one card: an immediate showdown (❓3: even with a
    /// caller).
    fn reveal(&mut self) {
        self.events.clear();
        let chosen = std::mem::take(&mut self.placed);
        self.reset_choices();
        for (i, card) in chosen.iter().enumerate() {
            if let Some(card) = *card {
                self.take_from_hand(Seat(i as u8), card);
                self.played.push(card);
            }
        }
        for i in 0..self.tracked.len() {
            let to = self.next_active(Seat(i as u8)).index();
            if self.tracked[i].is_some() && self.tracked[i] == chosen[to] {
                self.tracked[i] = None;
            }
        }
        let cards: Vec<Rank> = chosen.iter().flatten().copied().collect();
        let counts = count_by_rank(&cards);
        let caller = self
            .active()
            .into_iter()
            .filter_map(|s| chosen[s.index()].map(|c| (c, s)))
            .filter(|(c, _)| counts[c.index()] == 1)
            .min()
            .map(|(_, s)| s);
        self.events.push(Event::CardsPlayed {
            cards: chosen,
            caller,
        });
        let last_card = self
            .active()
            .iter()
            .all(|s| self.hands[s.index()].len() <= 1);
        if last_card {
            self.showdown(caller, None);
            return;
        }
        match caller {
            Some(seat) => {
                self.caller = Some(seat);
                self.phase = Phase::Call;
            }
            None => self.phase = Phase::Play,
        }
    }

    /// [R] Each player still in passes the chosen card to the next player
    /// still in on the left.
    fn resolve_passes(&mut self) {
        let chosen = std::mem::take(&mut self.placed);
        self.reset_choices();
        let n = usize::from(self.players);
        self.received = vec![None; n];
        for (i, card) in chosen.iter().enumerate() {
            if let Some(card) = *card {
                self.take_from_hand(Seat(i as u8), card);
            }
        }
        for (i, card) in chosen.iter().enumerate() {
            if let Some(card) = *card {
                let to = self.next_active(Seat(i as u8));
                self.hands[to.index()].push(card);
                self.received[to.index()] = Some(card);
            }
        }
        self.tracked = chosen.clone();
        self.passed = chosen;
        self.events.push(Event::Passed);
        self.phase = Phase::Play;
    }

    /// [R] The best hand takes the pot (tied best hands split it, ❓4 the
    /// odd chips staying in the pot); the worst hand antes 5 on the next
    /// hand. On the last hand the penalty goes to the winners instead.
    fn showdown(&mut self, caller: Option<Seat>, best: Option<Kind>) {
        let kind = best.unwrap_or(Kind::Clean);
        let active = self.active();
        let key = |s: &Seat| strength(&self.hands[s.index()], kind);
        let top = active.iter().map(key).min().unwrap_or_default();
        let bottom = active.iter().map(key).max().unwrap_or_default();
        let winners: Vec<Seat> = active.iter().copied().filter(|s| key(s) == top).collect();
        // ❓5: when every hand ties, nobody loses.
        let losers: Vec<Seat> = if top == bottom {
            Vec::new()
        } else {
            active
                .iter()
                .copied()
                .filter(|s| key(s) == bottom)
                .collect()
        };
        let last = self.hand_no >= self.hands_total;
        if last {
            for &loser in &losers {
                self.pay(loser, PENALTY);
            }
        }
        let share = self.pot / winners.len().max(1) as u32;
        for &w in &winners {
            self.chips[w.index()] += share as i32;
            self.pot -= share;
        }
        self.events.push(Event::Showdown {
            caller,
            best,
            hands: self
                .seats()
                .map(|s| (!self.folded[s.index()]).then(|| self.hands[s.index()].clone()))
                .collect(),
            winners: winners.clone(),
            losers: losers.clone(),
            share,
        });
        // ❓6: tied winners: the first from the leader leads.
        if let Some(&first) = winners
            .iter()
            .min_by_key(|s| (s.0 + self.players - self.leader.0) % self.players)
        {
            self.leader = first;
        }
        self.losers = Some(losers);
        self.end_hand();
    }

    fn choosing(&self) -> bool {
        matches!(self.phase, Phase::Play | Phase::Pass)
    }
}

impl Game for Octopus {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-octopus",
            name: "Octopus",
            min_players: 4,
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
            undealt: Vec::new(),
            hands: Vec::new(),
            folded: vec![false; n],
            // [R] The dealer declares first on the first hand.
            leader: Seat(0),
            turn: None,
            placed: vec![None; n],
            passed: vec![None; n],
            received: vec![None; n],
            tracked: vec![None; n],
            played: Vec::new(),
            caller: None,
            phase: Phase::Declare,
            pot: 0,
            chips: vec![0; n],
            losers: None,
            events: Vec::new(),
        };
        s.start_hand();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Declare => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Play | Phase::Pass => Pending(
                s.active()
                    .into_iter()
                    .filter(|x| s.placed[x.index()].is_none())
                    .collect(),
            ),
            Phase::Call => s.caller.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    /// While cards are chosen nobody learns who has already chosen.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        if !s.choosing() {
            return Self::pending(s);
        }
        match viewer {
            Viewer::Seat(seat) if Self::pending(s).contains(seat) => Pending::one(seat),
            _ => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if !Self::pending(s).contains(seat) {
            return Vec::new();
        }
        let hand = || {
            let mut cards = s.hands[seat.index()].clone();
            cards.sort_unstable();
            cards.dedup();
            cards
        };
        match s.phase {
            Phase::Declare => vec![Action::Stay, Action::Fold],
            Phase::Play => hand()
                .into_iter()
                .map(|card| Action::Play { card })
                .collect(),
            Phase::Pass => hand()
                .into_iter()
                .map(|card| Action::Pass { card })
                .collect(),
            Phase::Call => vec![
                Action::CallPass,
                Action::Show { best: Kind::Set },
                Action::Show { best: Kind::Clean },
            ],
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new(if s.choosing() {
                "Tu as déjà choisi ta carte."
            } else {
                "Ce n'est pas ton tour."
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Stay => s.declare(seat, false),
            Action::Fold => s.declare(seat, true),
            Action::Play { card } | Action::Pass { card } => {
                s.placed[seat.index()] = Some(card);
                let done = s.active().iter().all(|x| s.placed[x.index()].is_some());
                if done {
                    if s.phase == Phase::Play {
                        s.reveal();
                    } else {
                        s.resolve_passes();
                    }
                }
            }
            Action::CallPass => {
                s.events.clear();
                s.events.push(Event::PassCalled { seat });
                s.caller = None;
                s.phase = Phase::Pass;
            }
            Action::Show { best } => {
                s.events.clear();
                s.caller = None;
                s.showdown(Some(seat), Some(best));
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
