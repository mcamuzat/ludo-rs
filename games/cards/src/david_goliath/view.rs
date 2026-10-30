use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::heuristics::current_points;
use super::{Action, Card, RoundLine, State, TakenTrick};

/// What one seat sees: its own hand, every hand size, the table, the cards
/// won by each player and the scores. Never another hand nor the seed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    /// By colour, then highest first (empty for a spectator).
    pub hand: Vec<Card>,
    pub hand_counts: Vec<u8>,
    /// The trick being played, in order.
    pub trick: Vec<(Seat, Card)>,
    pub last_trick: Option<TakenTrick>,
    /// Cards won this round, by seat, sorted by colour then value.
    pub won: Vec<Vec<Card>>,
    /// What those cards are worth now.
    pub points: Vec<i32>,
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
        let won = s
            .won
            .iter()
            .map(|w| {
                let mut w = w.clone();
                w.sort();
                w
            })
            .collect();
        Self {
            me,
            hand,
            hand_counts: s.hands.iter().map(|h| h.len() as u8).collect(),
            trick: s.trick.clone(),
            last_trick: s.last_trick.clone(),
            won,
            points: current_points(s),
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
