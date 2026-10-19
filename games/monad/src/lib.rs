//! Monad (Sid Sackson, 3M 1970), sources and rules in RULES.md: trade, buy
//! and leap from Units up to Monads; the first to gather enough wins.

mod cards;
mod determinize;
mod heuristics;
mod rules;
mod view;

pub use cards::{Card, Color, Rank, Target};
pub use view::{EventView, View};

use std::collections::VecDeque;

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// [M] The Monads on the table at the start.
pub const MONADS: u8 = 10;

/// Site limit: the game stops after this many table rounds without a Monad.
pub const ROUNDS_WITHOUT_MONAD: u32 = 40;

/// [M] Monads needed to win: 5 with two players, 4 with three, 3 with four.
pub fn monads_to_win(players: u8) -> u8 {
    match players {
        2 => 5,
        3 => 4,
        _ => 3,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {}

/// Cards and colours are listed in their sorted order, so that a move has a
/// single spelling.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Trade { cards: [Card; 2] },
    Buy { target: Target, cards: Vec<Card> },
    Leap { colors: Vec<Color> },
    TakeBonus,
    DeclineBonus,
    EndTurn,
    Draw,
    Flip,
    Pass,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    /// A trade, buy or leap was made this turn: no draw, flip or pass.
    pub transacted: bool,
    /// [M] A trade or buy was made this turn: leaping is allowed.
    pub prepared: bool,
    /// [M] Only one bonus per turn.
    pub bonus_used: bool,
    /// The rank of the cards just traded, while their bonus waits for a
    /// decision.
    pub bonus_offer: Option<Rank>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub seat: Seat,
    pub action: Action,
    /// Cards taken from the columns, in sight of everyone.
    pub gained: Vec<Card>,
    pub monad: bool,
    /// The Unit taken from the face-down deck: seen by its taker only.
    pub drawn: Option<Color>,
}

impl Event {
    fn new(seat: Seat, action: Action) -> Self {
        Event {
            seat,
            action,
            gained: Vec::new(),
            monad: false,
            drawn: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    /// Sorted.
    pub hands: Vec<Vec<Card>>,
    /// [M] The colour of each seat's Identity card: its wild cards.
    pub identities: Vec<Color>,
    pub monads: Vec<u8>,
    pub monad_stock: u8,
    /// Bi, Tri, Quad and Quint columns; the front card is the top one.
    pub columns: [VecDeque<Color>; 4],
    /// Face-up Units, the last one on top.
    pub discard: Vec<Color>,
    /// Face-down Units, the last one on top.
    pub deck: Vec<Color>,
    pub current: Seat,
    pub turn: Turn,
    pub this_turn: Vec<Event>,
    pub last_turn: Vec<Event>,
    /// Consecutive turns that were a pass.
    pub passes: u8,
    pub turns_since_monad: u32,
    pub winner: Option<Seat>,
    /// The game stopped on a site limit, without a winner.
    pub stalled: bool,
    /// Moves applied so far (the AI's simulation horizon).
    pub moves: u32,
}

impl State {
    pub fn over(&self) -> bool {
        self.winner.is_some() || self.stalled
    }

    /// Ranks by Monads, most first; equal counts tie. A winner is the only
    /// seat with the target, so it comes first.
    pub fn ranks(&self) -> Vec<u8> {
        self.monads
            .iter()
            .map(|&mine| {
                let more = self.monads.iter().filter(|&&m| m > mine).count();
                u8::try_from(more).expect("at most 4 seats")
            })
            .collect()
    }

    pub fn column(&self, rank: Rank) -> Option<&VecDeque<Color>> {
        rank.column().map(|i| &self.columns[i])
    }

    /// Whether the top of `target` can be taken now.
    pub fn available(&self, target: Target) -> bool {
        match target {
            Target::Card(rank) => self.column(rank).is_some_and(|c| !c.is_empty()),
            Target::Monad => self.monad_stock > 0,
        }
    }

    /// Cards handed to the table: Units on the discard, the others under
    /// their column, in sorted order.
    fn give_back(&mut self, seat: Seat, cards: &[Card]) {
        let hand = &mut self.hands[seat.index()];
        for card in cards {
            let at = hand
                .iter()
                .position(|c| c == card)
                .expect("legal: the card is held");
            hand.remove(at);
            match card.rank.column() {
                Some(i) => self.columns[i].push_back(card.color),
                None => self.discard.push(card.color),
            }
        }
    }

    /// Takes the top of `target` for `seat`; a Monad may win the game.
    fn gain(&mut self, seat: Seat, target: Target, event: &mut Event) {
        match target {
            Target::Card(rank) => {
                let i = rank.column().expect("Units are never taken from a column");
                let color = self.columns[i]
                    .pop_front()
                    .expect("legal: the column is not empty");
                let card = Card { rank, color };
                let hand = &mut self.hands[seat.index()];
                hand.push(card);
                hand.sort();
                event.gained.push(card);
            }
            Target::Monad => {
                self.monad_stock -= 1;
                self.monads[seat.index()] += 1;
                self.turns_since_monad = 0;
                event.monad = true;
                // [M] The first to gather the Monads needed wins at once.
                if self.monads[seat.index()] >= monads_to_win(self.players) {
                    self.winner = Some(seat);
                }
            }
        }
    }

    /// Ends the current turn: the turn passes to the left.
    fn end_turn(&mut self) {
        self.last_turn = std::mem::take(&mut self.this_turn);
        self.turn = Turn::default();
        self.current = Seat((self.current.0 + 1) % self.players);
        self.turns_since_monad += 1;
        if self.turns_since_monad >= ROUNDS_WITHOUT_MONAD * u32::from(self.players) {
            self.stalled = true;
        }
    }
}

pub struct Monad;

impl Game for Monad {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "monad",
            name: "Monad",
            min_players: 2,
            max_players: 4,
            hidden_info: true,
            clock: true,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let n = usize::from(players);
        // [M] Units in play: two, three or four of each colour.
        let mut units: Vec<Color> = Color::ALL
            .iter()
            .flat_map(|&c| std::iter::repeat_n(c, n))
            .collect();
        units.shuffle(&mut rng);
        let mut hands = vec![Vec::new(); n];
        for (i, color) in units.into_iter().enumerate() {
            hands[i % n].push(Card {
                rank: Rank::Unit,
                color,
            });
        }
        for hand in &mut hands {
            hand.sort();
        }
        let columns = std::array::from_fn(|_| {
            let mut column = Color::ALL.to_vec();
            column.shuffle(&mut rng);
            VecDeque::from(column)
        });
        // [M] Identity cards in play for each table size.
        let mut identities = match players {
            2 => Color::ALL.to_vec(),
            3 => vec![Color::Navy, Color::Sky, Color::Green],
            _ => vec![Color::Red, Color::Orange, Color::Navy, Color::Sky],
        };
        identities.shuffle(&mut rng);
        identities.truncate(n);
        let first = rng.gen::<u32>() % u32::from(players);
        State {
            players,
            hands,
            identities,
            monads: vec![0; n],
            monad_stock: MONADS,
            columns,
            discard: Vec::new(),
            deck: Vec::new(),
            current: Seat(u8::try_from(first).expect("first < players")),
            turn: Turn::default(),
            this_turn: Vec::new(),
            last_turn: Vec::new(),
            passes: 0,
            turns_since_monad: 0,
            winner: None,
            stalled: false,
            moves: 0,
        }
    }

    fn pending(s: &State) -> Pending {
        if s.over() {
            Pending::none()
        } else {
            Pending::one(s.current)
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.over() || seat != s.current {
            return Vec::new();
        }
        if s.turn.bonus_offer.is_some() {
            return vec![Action::TakeBonus, Action::DeclineBonus];
        }
        let mut actions: Vec<Action> = rules::trades(s, seat)
            .into_iter()
            .map(|cards| Action::Trade { cards })
            .collect();
        actions.extend(
            rules::buys(s, seat)
                .into_iter()
                .map(|(target, cards)| Action::Buy { target, cards }),
        );
        actions.extend(
            rules::leaps(s, seat)
                .into_iter()
                .map(|colors| Action::Leap { colors }),
        );
        if s.turn.transacted {
            actions.push(Action::EndTurn);
        } else if !s.deck.is_empty() {
            actions.push(Action::Draw);
        } else if !s.discard.is_empty() {
            actions.push(Action::Flip);
        }
        if !s.turn.transacted && rules::may_pass(s, seat) {
            actions.push(Action::Pass);
        }
        actions
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.over() {
            return Err(Illegal::new("La partie est finie."));
        }
        if seat != s.current {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis maintenant."));
        }
        s.moves += 1;
        let mut event = Event::new(seat, action.clone());
        match action {
            Action::EndTurn => {}
            Action::Draw => {
                let color = s.deck.pop().expect("legal: the deck is not empty");
                s.hands[seat.index()].push(Card {
                    rank: Rank::Unit,
                    color,
                });
                s.hands[seat.index()].sort();
                event.drawn = Some(color);
            }
            // [M] Turned face down as it is: the first Unit discarded is
            // the first drawn.
            Action::Flip => s.deck = s.discard.drain(..).rev().collect(),
            Action::Pass => {}
            Action::Trade { cards } => {
                let (target, joker) =
                    rules::trade_target(s, seat, *cards).expect("legal: the pair trades");
                s.give_back(seat, cards);
                s.gain(seat, target, &mut event);
                s.turn.bonus_offer = rules::bonus_of(s, *cards, joker);
            }
            Action::Buy { target, cards } => {
                s.give_back(seat, cards);
                s.gain(seat, *target, &mut event);
            }
            Action::Leap { colors } => {
                let units: Vec<Card> = colors
                    .iter()
                    .map(|&color| Card {
                        rank: Rank::Unit,
                        color,
                    })
                    .collect();
                s.give_back(seat, &units);
                let rank = rules::leap_rank(colors.len()).expect("legal: 4 to 6 Units");
                s.gain(seat, Target::Card(rank), &mut event);
            }
            // [M] The top card of each lower set, and a Unit from the deck.
            Action::TakeBonus => {
                let rank = s.turn.bonus_offer.take().expect("legal: a bonus waits");
                for &lower in &Rank::ALL[1..rank.index()] {
                    if s.available(Target::Card(lower)) {
                        s.gain(seat, Target::Card(lower), &mut event);
                    }
                }
                if let Some(color) = s.deck.pop() {
                    s.hands[seat.index()].push(Card {
                        rank: Rank::Unit,
                        color,
                    });
                    s.hands[seat.index()].sort();
                    event.drawn = Some(color);
                }
                s.turn.bonus_used = true;
            }
            Action::DeclineBonus => s.turn.bonus_offer = None,
        }
        if matches!(action, Action::Trade { .. } | Action::Buy { .. }) {
            s.turn.prepared = true;
        }
        if matches!(
            action,
            Action::Trade { .. } | Action::Buy { .. } | Action::Leap { .. }
        ) {
            s.turn.transacted = true;
        }
        let whole_turn = matches!(action, Action::Draw | Action::Flip | Action::Pass);
        if *action == Action::Pass {
            s.passes += 1;
        } else {
            s.passes = 0;
        }
        if *action != Action::EndTurn {
            s.this_turn.push(event);
        }
        if (whole_turn || *action == Action::EndTurn) && !s.over() {
            s.end_turn();
        }
        // Site limit: a full round of passes changes nothing any more.
        if s.passes >= s.players {
            s.stalled = true;
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.over().then(|| Outcome::Ranked(s.ranks()))
    }

    /// Decline a bonus, end a turn already played, draw, flip or pass;
    /// failing all that, the first legal move.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        [
            Action::DeclineBonus,
            Action::EndTurn,
            Action::Draw,
            Action::Flip,
            Action::Pass,
        ]
        .into_iter()
        .find(|a| legal.contains(a))
        .or_else(|| legal.into_iter().next())
    }
}
