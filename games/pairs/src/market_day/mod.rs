//! Market Day (James Ernest, Cheapass Games 2018): each round everyone
//! brings one card to market; repeated ranks spoil, and five special cards
//! bend the rules. Rules and their sources: RULES.md, where [M] is the
//! monthly sheet.

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

use crate::deck::{full_deck, Rank};

pub struct MarketDay;

const TINKER: u8 = 1;
const KNIGHT: u8 = 2;
const ROYAL: u8 = 3;
const RAIN: u8 = 7;
const BANDIT: u8 = 9;

/// [M] Six cards each; ten at 2–3 players.
pub fn hand_size(players: u8) -> usize {
    if players <= 3 {
        10
    } else {
        6
    }
}

/// [M] At 2–3 players each plays two cards a round (twice around).
pub fn cards_each(players: u8) -> usize {
    if players <= 3 {
        2
    } else {
        1
    }
}

/// [M] Six hands, or one per player, whichever is greater.
pub fn hands_in_game(players: u8) -> u8 {
    players.max(6)
}

/// What became of a card played in a round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fate {
    Scored,
    Spoiled,
    Killed,
}

/// [M] The fate of each card of a round, in play order, before the Tinker:
/// remove spoiled cards first, then Royals, Rain and Knights.
pub fn resolve(trick: &[Rank]) -> Vec<Fate> {
    let v = |i: usize| trick[i].value();
    let count = |rank: u8| trick.iter().filter(|c| c.value() == rank).count();
    let mut fates = vec![Fate::Scored; trick.len()];
    for (i, fate) in fates.iter_mut().enumerate() {
        let rank = v(i);
        let earlier = (0..i).any(|j| v(j) == rank);
        let later = (i + 1..trick.len()).any(|j| v(j) == rank);
        *fate = match rank {
            // [M] Royals are never spoiled; a second Knight does not spoil
            // the first.
            ROYAL | KNIGHT => Fate::Scored,
            // [M] The first Bandit scores, the others are spoiled.
            BANDIT if earlier => Fate::Spoiled,
            BANDIT => Fate::Scored,
            // [M] Otherwise only the last of a repeated rank scores.
            _ if later => Fate::Spoiled,
            _ => Fate::Scored,
        };
    }
    // [M] A lone Rain spoils everything except Royals (and itself).
    if count(RAIN) == 1 {
        for (i, fate) in fates.iter_mut().enumerate() {
            if !matches!(v(i), ROYAL | RAIN) {
                *fate = Fate::Spoiled;
            }
        }
    }
    // [M] Each Knight kills the highest scoring card; never rank 3 or
    // lower. ❓ One after the other, in play order (at most one card of a
    // rank of 4 or more still scores, so there is never a tie).
    for k in 0..trick.len() {
        if v(k) != KNIGHT || fates[k] != Fate::Scored {
            continue;
        }
        let victim = (0..trick.len())
            .filter(|&i| fates[i] == Fate::Scored && v(i) > ROYAL)
            .max_by_key(|&i| (v(i), i));
        if let Some(i) = victim {
            fates[i] = Fate::Killed;
        }
    }
    fates
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Play {
        card: Rank,
    },
    /// Tinker: trade places with the card at `with` in the round.
    Swap {
        with: usize,
    },
    /// Tinker: no trade.
    Keep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Play,
    /// The Tinker's player decides on a trade.
    Tinker,
    Over,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Dealt {
        hand: u8,
        dealer: Seat,
    },
    Played {
        seat: Seat,
        card: Rank,
    },
    Swapped {
        seat: Seat,
        with: Seat,
        card: Rank,
    },
    /// Each card with its owner after any trade, and what became of it.
    TrickScored {
        cards: Vec<(Seat, Rank, Fate)>,
        gained: Vec<u16>,
        scores: Vec<u16>,
    },
    HandOver {
        scores: Vec<u16>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub hand_no: u8,
    /// For each hand's shuffle. Never in a view.
    pub seed: u64,
    pub dealer: Seat,
    /// Cards not dealt. Never in a view.
    pub undealt: Vec<Rank>,
    pub hands: Vec<Vec<Rank>>,
    /// The round in progress, in play order.
    pub trick: Vec<(Seat, Rank)>,
    /// Cards of earlier rounds this hand, face up.
    pub discard: Vec<Rank>,
    /// Who leads the round in progress.
    pub leader: Seat,
    pub scores: Vec<u16>,
    pub phase: Phase,
    pub turn: Seat,
    /// Since the last action.
    pub events: Vec<Event>,
}

/// One independent stream per hand.
fn mix(seed: u64, hand: u8) -> u64 {
    seed ^ u64::from(hand).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl State {
    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    pub fn playable(&self, seat: Seat) -> Vec<Rank> {
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        cards
    }

    /// The scoring Tinker of the round, if any (index in the round).
    pub fn tinker(&self) -> Option<usize> {
        let cards: Vec<Rank> = self.trick.iter().map(|&(_, c)| c).collect();
        let fates = resolve(&cards);
        (0..cards.len()).find(|&i| cards[i].value() == TINKER && fates[i] == Fate::Scored)
    }

    /// ❓ The cards the Tinker may trade with: another player's card that
    /// still scores, never a Royal.
    pub fn swaps(&self) -> Vec<usize> {
        let Some(t) = self.tinker() else {
            return Vec::new();
        };
        let cards: Vec<Rank> = self.trick.iter().map(|&(_, c)| c).collect();
        let fates = resolve(&cards);
        let owner = self.trick[t].0;
        (0..cards.len())
            .filter(|&i| {
                fates[i] == Fate::Scored && cards[i].value() != ROYAL && self.trick[i].0 != owner
            })
            .collect()
    }

    /// [M] Six (or ten) cards each; the player on the dealer's left leads.
    fn deal(&mut self, dealer: Seat) {
        let n = usize::from(self.players);
        self.hand_no += 1;
        self.undealt = full_deck();
        self.undealt
            .shuffle(&mut ChaCha8Rng::seed_from_u64(mix(self.seed, self.hand_no)));
        self.hands = vec![Vec::new(); n];
        for _ in 0..hand_size(self.players) {
            for hand in &mut self.hands {
                hand.extend(self.undealt.pop());
            }
        }
        self.trick.clear();
        self.discard.clear();
        self.dealer = dealer;
        self.leader = self.next(dealer);
        self.turn = self.leader;
        self.events.push(Event::Dealt {
            hand: self.hand_no,
            dealer,
        });
    }

    fn play(&mut self, seat: Seat, card: Rank) {
        let hand = &mut self.hands[seat.index()];
        if let Some(at) = hand.iter().position(|&c| c == card) {
            hand.remove(at);
        }
        self.trick.push((seat, card));
        self.events.push(Event::Played { seat, card });
        let full = self.trick.len() == usize::from(self.players) * cards_each(self.players);
        if full || self.hands.iter().all(Vec::is_empty) {
            if self.swaps().is_empty() {
                self.score_trick();
            } else {
                self.phase = Phase::Tinker;
                self.turn = self.trick[self.tinker().expect("a Tinker")].0;
            }
        } else {
            self.turn = self.next(seat);
        }
    }

    /// [M] The Tinker trades places: each player now owns the other card.
    fn swap(&mut self, with: usize) {
        let t = self.tinker().expect("a Tinker");
        let (seat, other) = (self.trick[t].0, self.trick[with].0);
        self.trick[t].0 = other;
        self.trick[with].0 = seat;
        self.events.push(Event::Swapped {
            seat,
            with: other,
            card: self.trick[with].1,
        });
    }

    /// [M] Score the round; the top scorer leads the next one (a tie keeps
    /// the leader). The hand ends once every card is played.
    fn score_trick(&mut self) {
        let cards: Vec<Rank> = self.trick.iter().map(|&(_, c)| c).collect();
        let fates = resolve(&cards);
        let mut gained = vec![0u16; usize::from(self.players)];
        for (&(seat, card), fate) in self.trick.iter().zip(&fates) {
            if *fate == Fate::Scored {
                gained[seat.index()] += u16::from(card.value());
            }
        }
        for (score, g) in self.scores.iter_mut().zip(&gained) {
            *score += g;
        }
        self.events.push(Event::TrickScored {
            cards: self
                .trick
                .iter()
                .zip(&fates)
                .map(|(&(seat, card), &fate)| (seat, card, fate))
                .collect(),
            gained: gained.clone(),
            scores: self.scores.clone(),
        });
        let best = gained.iter().copied().max().unwrap_or(0);
        if gained.iter().filter(|&&g| g == best).count() == 1 {
            self.leader = Seat(gained.iter().position(|&g| g == best).expect("a best") as u8);
        }
        self.discard.extend(cards);
        self.trick.clear();
        self.phase = Phase::Play;
        self.turn = self.leader;
        if self.hands.iter().all(Vec::is_empty) {
            self.events.push(Event::HandOver {
                scores: self.scores.clone(),
            });
            if self.hand_no >= hands_in_game(self.players) {
                self.phase = Phase::Over;
            } else {
                let dealer = self.next(self.dealer);
                self.deal(dealer);
            }
        }
    }
}

impl Game for MarketDay {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-market-day",
            name: "Market Day",
            min_players: 2,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            hand_no: 0,
            seed,
            dealer: Seat(0),
            undealt: Vec::new(),
            hands: vec![Vec::new(); n],
            trick: Vec::new(),
            discard: Vec::new(),
            leader: Seat(0),
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
            Phase::Play | Phase::Tinker => Pending::one(s.turn),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != seat {
            return Vec::new();
        }
        match s.phase {
            Phase::Play => s
                .playable(seat)
                .into_iter()
                .map(|card| Action::Play { card })
                .collect(),
            Phase::Tinker => s
                .swaps()
                .into_iter()
                .map(|with| Action::Swap { with })
                .chain([Action::Keep])
                .collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != seat {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        s.events.clear();
        match *action {
            Action::Play { card } => s.play(seat, card),
            Action::Swap { with } => {
                s.swap(with);
                s.score_trick();
            }
            Action::Keep => s.score_trick(),
        }
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
