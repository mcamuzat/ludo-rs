use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{pile_cards, Action, Card, Color, RoundLine, State, TakenTrick};

/// A pile open in the current trick.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Pile {
    pub color: Color,
    /// In the order played.
    pub cards: Vec<(Seat, Card)>,
}

/// What one seat sees: its own hand, every hand size, the table, the cards
/// won and put down by each player, and the scores. Never another hand nor
/// the seed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    /// By colour, then highest first (empty for a spectator).
    pub hand: Vec<Card>,
    pub hand_counts: Vec<u8>,
    /// The piles of the current trick, in the order opened.
    pub piles: Vec<Pile>,
    /// The passes of the current trick: who passed, showing which card.
    pub passes: Vec<(Seat, Card)>,
    pub last_trick: Option<TakenTrick>,
    /// Cards won this round, by seat and colour (red, blue, green).
    pub won: Vec<[u8; 3]>,
    /// Cards put down to pass this round, by seat.
    pub passed: Vec<u8>,
    pub scores: Vec<i32>,
    /// From 1, for display.
    pub round: u32,
    pub rounds: u32,
    pub dealer: Seat,
    pub turn: Option<Seat>,
    pub last_round: Option<Vec<RoundLine>>,
    /// The viewer's legal actions (empty when it is not its turn).
    pub legal: Vec<Action>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < s.hands.len() => Some(seat),
            _ => None,
        };
        let mut hand = me.map(|m| s.hands[m.index()].clone()).unwrap_or_default();
        hand.sort_by_key(|c| (c.color, std::cmp::Reverse(c.value)));
        Self {
            me,
            hand,
            hand_counts: s.hands.iter().map(|h| h.len() as u8).collect(),
            piles: s
                .pile_colors()
                .into_iter()
                .map(|color| Pile {
                    color,
                    cards: pile_cards(&s.trick, color),
                })
                .collect(),
            passes: s
                .trick
                .iter()
                .filter_map(|&(seat, a)| match a {
                    Action::Pass { card } => Some((seat, card)),
                    Action::Play { .. } => None,
                })
                .collect(),
            last_trick: s.last_trick.clone(),
            won: s.won.clone(),
            passed: s.passed.clone(),
            scores: s.scores.clone(),
            round: s.round + 1,
            rounds: s.rounds(),
            dealer: s.dealer,
            turn: s.turn,
            last_round: s.last_round.clone(),
            legal: me.map(|m| s.moves(m)).unwrap_or_default(),
        }
    }
}
