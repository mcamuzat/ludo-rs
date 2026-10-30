//! David & Goliath (Reinhard Staupe): the highest card of a trick takes the
//! others, the lowest takes the highest. One or two cards of a colour score
//! their value, more score one point each. Rules and their sources:
//! RULES.md, where [R] is the English rule.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::View;

use std::fmt;

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::deal_evenly;
use crate::standings;

pub struct DavidGoliath;

pub const MIN_PLAYERS: u8 = 3;
pub const MAX_PLAYERS: u8 = 6;
/// [R] Fifteen cards in each hand.
pub const HAND: usize = 15;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

/// [R] In the order of the final count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Violet,
}

impl Color {
    pub const ALL: [Color; 5] = [
        Color::Red,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Violet,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub color: Color,
    /// 1 to 18.
    pub value: u8,
}

impl fmt::Display for Card {
    /// `g9`: the colour's initial (r, y, g, b, v) and the value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letter = ['r', 'y', 'g', 'b', 'v'][self.color.index()];
        write!(f, "{letter}{}", self.value)
    }
}

/// [R] 1 to 9 at three players, to 12 at four, 15 at five, 18 at six: the
/// pack always deals fifteen cards each.
pub fn pack(players: u8) -> Vec<Card> {
    let highest = 3 * players;
    Color::ALL
        .iter()
        .flat_map(|&color| (1..=highest).map(move |value| Card { color, value }))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Play { card: Card },
}

impl Action {
    pub fn card(self) -> Card {
        match self {
            Action::Play { card } => card,
        }
    }
}

/// The last trick: the cards in the order played, who had the highest card
/// (and took the rest, and leads) and who had the lowest (and took the
/// highest).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TakenTrick {
    pub cards: Vec<(Seat, Card)>,
    pub highest: Seat,
    pub lowest: Seat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundLine {
    pub points: i32,
    /// The total after the round.
    pub total: i32,
}

/// [R] One colour: the face values of one or two cards, one point a card
/// beyond.
pub fn color_points(cards: &[Card]) -> i32 {
    if cards.len() <= 2 {
        cards.iter().map(|c| i32::from(c.value)).sum()
    } else {
        cards.len() as i32
    }
}

/// [R] Each colour counted apart, then added up.
pub fn hand_points(won: &[Card]) -> i32 {
    Color::ALL
        .iter()
        .map(|&color| {
            let cards: Vec<Card> = won.iter().copied().filter(|c| c.color == color).collect();
            color_points(&cards)
        })
        .sum()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub players: u8,
    /// Every deal comes from it: never in a view.
    pub seed: u64,
    /// From 0.
    pub round: u32,
    pub dealer: Seat,
    pub hands: Vec<Vec<Card>>,
    /// The trick being played, in order.
    pub trick: Vec<(Seat, Card)>,
    pub last_trick: Option<TakenTrick>,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// Cards won this round, by seat, in view of all.
    pub won: Vec<Vec<Card>>,
    /// `void[seat][colour]`: the seat did not follow this colour this
    /// round, so it holds none (public).
    pub void: Vec<[bool; 5]>,
    pub scores: Vec<i32>,
    /// The previous round's scoring.
    pub last_round: Option<Vec<RoundLine>>,
}

impl State {
    /// [R] The led colour must be followed when possible.
    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.turn != Some(seat) {
            return Vec::new();
        }
        let hand = &self.hands[seat.index()];
        let led = self.trick.first().map(|&(_, c)| c.color);
        let follows = |c: &Card| led.is_none_or(|color| c.color == color);
        let can_follow = hand.iter().any(follows);
        hand.iter()
            .filter(|c| !can_follow || follows(c))
            .map(|&card| Action::Play { card })
            .collect()
    }

    /// [R] As many rounds as players.
    pub fn rounds(&self) -> u32 {
        u32::from(self.players)
    }

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// A fresh round from the round's own stream of the seed, so that a log
    /// replays identically. ❓ [R] names only a starting player: the player
    /// at the dealer's left leads.
    fn start_round(&mut self) {
        let stream = self.seed ^ u64::from(self.round + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.hands = deal_evenly(pack(self.players), self.players, stream).hands;
        self.trick.clear();
        // The new round is not led by the last trick's taker.
        self.last_trick = None;
        self.won = vec![Vec::new(); usize::from(self.players)];
        self.void = vec![[false; 5]; usize::from(self.players)];
        self.turn = Some(self.next(self.dealer));
    }

    fn act(&mut self, seat: Seat, card: Card) {
        if let Some(&(_, led)) = self.trick.first() {
            if card.color != led.color {
                self.void[seat.index()][led.color.index()] = true;
            }
        }
        self.hands[seat.index()].retain(|&c| c != card);
        self.trick.push((seat, card));
        if self.trick.len() < usize::from(self.players) {
            self.turn = Some(self.next(seat));
        } else {
            self.end_trick();
        }
    }

    /// [R] Highest and lowest by value whatever the colour, the last played
    /// winning a tie. The lowest takes the highest card, the highest takes
    /// the rest and leads. ❓ When all values are equal, the last card is
    /// both, and its player takes everything.
    fn end_trick(&mut self) {
        let cards = std::mem::take(&mut self.trick);
        // `max_by_key`/`min_by_key` keep the last of equal elements and
        // the first of equal elements respectively: search the reversed
        // order for the lowest.
        let hi = (0..cards.len())
            .max_by_key(|&i| cards[i].1.value)
            .expect("a full trick");
        let lo = (0..cards.len())
            .rev()
            .min_by_key(|&i| cards[i].1.value)
            .expect("a full trick");
        let (highest, lowest) = (cards[hi].0, cards[lo].0);
        for (i, &(_, card)) in cards.iter().enumerate() {
            let taker = if i == hi { lowest } else { highest };
            self.won[taker.index()].push(card);
        }
        self.last_trick = Some(TakenTrick {
            cards,
            highest,
            lowest,
        });
        if self.hands.iter().all(Vec::is_empty) {
            self.end_round();
        } else {
            self.turn = Some(highest);
        }
    }

    fn end_round(&mut self) {
        let lines = (0..usize::from(self.players))
            .map(|i| {
                let points = hand_points(&self.won[i]);
                self.scores[i] += points;
                RoundLine {
                    points,
                    total: self.scores[i],
                }
            })
            .collect();
        self.last_round = Some(lines);
        if self.round + 1 >= self.rounds() {
            self.turn = None;
            return;
        }
        self.round += 1;
        // ❓ The deal passes to the left.
        self.dealer = self.next(self.dealer);
        self.start_round();
    }
}

impl Game for DavidGoliath {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cards-david-goliath",
            name: "David & Goliath",
            min_players: MIN_PLAYERS,
            max_players: MAX_PLAYERS,
            hidden_info: true,
            clock: true,
        }
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&players) {
            return Err(Illegal::new(format!(
                "David & Goliath se joue de {MIN_PLAYERS} à {MAX_PLAYERS} joueurs."
            )));
        }
        Ok(())
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            config: config.clone(),
            players,
            seed,
            round: 0,
            // ❓ [R] does not say who deals first: seat 0.
            dealer: Seat(0),
            hands: vec![Vec::new(); n],
            trick: Vec::new(),
            last_trick: None,
            turn: None,
            won: vec![Vec::new(); n],
            void: vec![[false; 5]; n],
            scores: vec![0; n],
            last_round: None,
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.turn.is_none() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        let card = action.card();
        if !s.hands[seat.index()].contains(&card) {
            return Err(Illegal::new("Cette carte n'est pas dans ta main."));
        }
        if !s.moves(seat).contains(action) {
            return Err(Illegal::new("Il faut fournir la couleur demandée."));
        }
        s.act(seat, card);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.turn
            .is_none()
            .then(|| Outcome::Ranked(standings(&s.scores)))
    }

    /// The lowest legal card.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        s.moves(seat)
            .into_iter()
            .min_by_key(|a| (a.card().value, a.card().color))
    }
}
