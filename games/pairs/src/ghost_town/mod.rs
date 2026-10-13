//! Ghost Town (James Ernest): a chip game where the pot is the bank. Rules
//! and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply};

pub struct GhostTown;

/// [R] Chips each player pays into the pot at the start of a hand.
pub const ANTE: u32 = 10;
/// [R] Chips paid into the pot to play a card from one's hand.
pub const PLAY_COST: u32 = 2;
/// [R] Cards dealt to each player.
pub const HAND: usize = 5;
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
    /// Setup: the hand card to put in the street.
    Place { card: Rank },
    /// Pay `PLAY_COST` and put a hand card face up in `to`'s house: one's
    /// own, or the left neighbour's.
    Play { card: Rank, to: Seat },
    /// Take the top card of the draw pile into one's house.
    Draw,
    /// Leave the hand.
    Fold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone chooses a card for the street, at the same time.
    Setup,
    Play,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Setup is over: `seat` had chosen `card` for the street.
    Placed {
        seat: Seat,
        card: Rank,
    },
    /// A card dealt to break a tie for the lowest card, then discarded.
    TieBreak {
        seat: Seat,
        card: Rank,
    },
    Starts {
        seat: Seat,
    },
    Played {
        seat: Seat,
        to: Seat,
        card: Rank,
    },
    /// A clean draw: `won` chips taken from the pot.
    Drew {
        seat: Seat,
        card: Rank,
        won: u32,
    },
    /// The card paired the house or the street: `seat` pays its rank and is out.
    Busted {
        seat: Seat,
        card: Rank,
    },
    Folded {
        seat: Seat,
    },
    /// `winner` was the last player in and took `pot`; `None`: the pot ran dry.
    HandEnded {
        winner: Option<Seat>,
        pot: u32,
    },
    Reshuffled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub hands_total: u8,
    /// The current hand, from 1.
    pub hand_no: u8,
    pub supply: Supply,
    /// For each hand's deal. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// Face-up cards of each seat. Never two of a rank, never a rank of the street.
    pub houses: Vec<Vec<Rank>>,
    /// Community cards: a house may not hold their ranks.
    pub street: Vec<Rank>,
    /// Setup: the card each seat has chosen, still in its hand.
    pub placed: Vec<Option<Rank>>,
    /// Folded or busted in the current hand.
    pub out: Vec<bool>,
    pub phase: Phase,
    /// Seat to move during play.
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

fn lowest_of(cards: &[(Seat, Rank)]) -> Vec<Seat> {
    let low = cards.iter().map(|&(_, card)| card).min();
    cards
        .iter()
        .filter(|&&(_, card)| Some(card) == low)
        .map(|&(seat, _)| seat)
        .collect()
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    /// Seats still in the hand, in seat order.
    pub fn active(&self) -> Vec<Seat> {
        self.seats().filter(|s| !self.out[s.index()]).collect()
    }

    /// The next seat still in the hand after `seat`, if there is another.
    pub fn left_of(&self, seat: Seat) -> Option<Seat> {
        (1..self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| !self.out[s.index()])
    }

    /// Per rank, the cards `observer` cannot place: not in the street, a
    /// house, the seen discards, nor its own hand.
    pub fn unlocated_for(&self, observer: Option<Seat>) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let own = observer
            .into_iter()
            .flat_map(|o| self.hands[o.index()].iter());
        for card in self
            .houses
            .iter()
            .flatten()
            .chain(&self.street)
            .chain(self.supply.seen())
            .chain(own)
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// Chips `seat` expects from a draw, from what it sees: the card's rank
    /// (at most the pot) if it is clean, minus its rank if it busts.
    pub fn draw_value(&self, seat: Seat) -> f64 {
        // With an empty draw pile the next card comes from the reshuffled
        // discards, not from the cards `seat` cannot place.
        let pool = if self.supply.deck_len() == 0 {
            self.supply.draw_pool()
        } else {
            self.unlocated_for(Some(seat))
        };
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 0.0;
        }
        let house = &self.houses[seat.index()];
        let value: f64 = Rank::all()
            .map(|rank| {
                let weight = f64::from(pool[rank.index()]);
                let chips = f64::from(rank.value());
                if has_rank(house, rank) || has_rank(&self.street, rank) {
                    -weight * chips
                } else {
                    weight * chips.min(f64::from(self.pot))
                }
            })
            .sum();
        value / f64::from(total)
    }

    fn draw(&mut self) -> Option<Rank> {
        let before = self.supply.shuffles();
        let card = self.supply.draw();
        if self.supply.shuffles() != before {
            self.events.push(Event::Reshuffled);
        }
        card
    }

    fn pay(&mut self, seat: Seat, chips: u32) {
        self.chips[seat.index()] -= chips as i32;
        self.pot += chips;
    }

    /// [R] Everyone antes and gets five cards. Leaves `events` alone.
    fn start_hand(&mut self) {
        let n = usize::from(self.players);
        self.hand_no += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.hand_no));
        self.hands = vec![Vec::new(); n];
        for _ in 0..HAND {
            for i in 0..n {
                if let Some(card) = self.supply.draw() {
                    self.hands[i].push(card);
                }
            }
        }
        self.houses = vec![Vec::new(); n];
        self.street = Vec::new();
        self.placed = vec![None; n];
        self.out = vec![false; n];
        self.phase = Phase::Setup;
        self.turn = None;
        for seat in self.seats().collect::<Vec<_>>() {
            self.pay(seat, ANTE);
        }
    }

    /// [R] The chosen cards make the street, duplicates are discarded, and
    /// the lowest card starts.
    fn reveal(&mut self) {
        self.events.clear();
        let mut shown: Vec<(Seat, Rank)> = Vec::new();
        for seat in self.seats().collect::<Vec<_>>() {
            let i = seat.index();
            let Some(card) = self.placed[i].take() else {
                continue;
            };
            let Some(at) = self.hands[i].iter().position(|&c| c == card) else {
                continue;
            };
            self.hands[i].remove(at);
            self.events.push(Event::Placed { seat, card });
            if has_rank(&self.street, card) {
                self.supply.discard([card]);
            } else {
                self.street.push(card);
            }
            shown.push((seat, card));
        }
        let mut tied = lowest_of(&shown);
        // [R] A tie is broken with cards from the deck, used only for that.
        while tied.len() > 1 {
            let mut round: Vec<(Seat, Rank)> = Vec::new();
            for &seat in &tied {
                // ❓5: the tie-break never reshuffles; an empty pile ends it.
                if self.supply.deck_len() == 0 {
                    break;
                }
                let Some(card) = self.draw() else {
                    break;
                };
                self.supply.discard([card]);
                self.events.push(Event::TieBreak { seat, card });
                round.push((seat, card));
            }
            if round.len() < tied.len() {
                break;
            }
            tied = lowest_of(&round);
        }
        let first = tied.first().copied().unwrap_or(Seat(0));
        self.events.push(Event::Starts { seat: first });
        self.phase = Phase::Play;
        self.turn = Some(first);
    }

    /// The house goes to the discards face up, the hand face down.
    fn leave(&mut self, seat: Seat) {
        let i = seat.index();
        self.out[i] = true;
        let house = std::mem::take(&mut self.houses[i]);
        self.supply.discard(house);
        let hand = std::mem::take(&mut self.hands[i]);
        self.supply.discard_unseen(hand);
    }

    /// After `seat` acted: the hand ends, or the turn passes left.
    fn settle(&mut self, seat: Seat) {
        if let [last] = self.active().as_slice() {
            // [R] The last player in collects the pot.
            let pot = std::mem::take(&mut self.pot);
            self.chips[last.index()] += pot as i32;
            return self.end_hand(Some(*last), pot);
        }
        if self.pot == 0 {
            // [R] The hand ends when the pot is empty.
            return self.end_hand(None, 0);
        }
        self.turn = self.left_of(seat);
    }

    fn end_hand(&mut self, winner: Option<Seat>, pot: u32) {
        self.events.push(Event::HandEnded { winner, pot });
        if self.hand_no >= self.hands_total {
            self.phase = Phase::Over;
            self.turn = None;
        } else {
            self.start_hand();
        }
    }

    /// What `seat` may do on its turn.
    fn play_actions(&self, seat: Seat) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.supply.can_draw() {
            actions.push(Action::Draw);
        }
        let house = &self.houses[seat.index()];
        let neighbour = self.left_of(seat);
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        for card in cards {
            if has_rank(&self.street, card) {
                continue;
            }
            if !has_rank(house, card) {
                // [R] No pair in a house, no card matching the street.
                actions.push(Action::Play { card, to: seat });
            } else if let Some(to) = neighbour {
                // [R] A rank I have at home and my left neighbour lacks.
                if !has_rank(&self.houses[to.index()], card) {
                    actions.push(Action::Play { card, to });
                }
            }
        }
        actions.push(Action::Fold);
        actions
    }

    fn play(&mut self, seat: Seat, action: &Action) {
        self.events.clear();
        match *action {
            // Not a play action: `apply` routes it elsewhere.
            Action::Place { .. } => return,
            Action::Play { card, to } => {
                let hand = &mut self.hands[seat.index()];
                if let Some(at) = hand.iter().position(|&c| c == card) {
                    hand.remove(at);
                    self.houses[to.index()].push(card);
                    self.pay(seat, PLAY_COST);
                }
                self.events.push(Event::Played { seat, to, card });
            }
            Action::Draw => {
                if let Some(card) = self.draw() {
                    let i = seat.index();
                    if has_rank(&self.houses[i], card) || has_rank(&self.street, card) {
                        // [R] Busted: out, and the card's rank goes to the pot.
                        self.pay(seat, u32::from(card.value()));
                        self.supply.discard([card]);
                        self.leave(seat);
                        self.events.push(Event::Busted { seat, card });
                    } else {
                        // [R] A clean draw wins the card's value from the pot
                        // (❓2: no more than the pot holds).
                        let won = u32::from(card.value()).min(self.pot);
                        self.pot -= won;
                        self.chips[i] += won as i32;
                        self.houses[i].push(card);
                        self.events.push(Event::Drew { seat, card, won });
                    }
                }
            }
            Action::Fold => {
                self.leave(seat);
                self.events.push(Event::Folded { seat });
            }
        }
        self.settle(seat);
    }
}

impl Game for GhostTown {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-ghost-town",
            name: "Ghost Town",
            min_players: 2,
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
        let mut s = State {
            players,
            hands_total,
            hand_no: 0,
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: Vec::new(),
            houses: Vec::new(),
            street: Vec::new(),
            placed: Vec::new(),
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
