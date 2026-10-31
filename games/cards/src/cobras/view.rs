use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{basket_points, Action, Card, Phase, Play, RoundLine, State, Suit, TrickResult, Who};

/// The two-player dummy as everyone sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DummyView {
    pub count: u8,
    /// Its next card, when the pile is face up.
    pub top: Option<Card>,
}

/// What one seat sees: its hand, every hand size, the baskets, the centre,
/// the line of triples, the table and the scores. Never another hand, the
/// cards set aside, the dummy's hidden cards, another seat's secret basket
/// card, nor the seed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    pub phase: Phase,
    /// By suit, then highest first (empty for a spectator).
    pub hand: Vec<Card>,
    /// The card the viewer picked for its basket, before the reveal.
    pub chosen: Option<Card>,
    pub hand_counts: Vec<u8>,
    pub dummy: Option<DummyView>,
    /// The cards that started the baskets, once revealed.
    pub revealed: Vec<Option<Card>>,
    pub baskets: Vec<u32>,
    /// What each basket would score if turned in now.
    pub basket_points: Vec<i32>,
    pub center: u32,
    pub triples: u32,
    pub triples_to_end: u32,
    /// The seat still holds its King Cobra.
    pub kings: Vec<bool>,
    pub trick: Vec<Play>,
    pub led: Option<Suit>,
    pub leader: Who,
    pub turn: Option<Who>,
    pub last_trick: Option<TrickResult>,
    pub first_leader: Seat,
    pub scores: Vec<i32>,
    pub target: i32,
    /// From 1, for display.
    pub round: u32,
    pub dealer: Seat,
    pub last_round: Option<Vec<RoundLine>>,
    /// The viewer's legal actions.
    pub legal: Vec<Action>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < s.hands.len() => Some(seat),
            _ => None,
        };
        let mut hand = me.map(|m| s.hands[m.index()].clone()).unwrap_or_default();
        hand.sort_by_key(|c| (c.suit, std::cmp::Reverse(c.value)));
        let dummy = s.has_dummy().then(|| DummyView {
            count: s.dummy.len() as u8,
            top: s
                .dummy_top_shown()
                .then(|| s.dummy.first().copied())
                .flatten(),
        });
        Self {
            me,
            phase: s.phase,
            hand,
            chosen: me.and_then(|m| s.chosen[m.index()]),
            hand_counts: s.hands.iter().map(|h| h.len() as u8).collect(),
            dummy,
            revealed: s.revealed.clone(),
            baskets: s.baskets.clone(),
            basket_points: s.baskets.iter().map(|&b| basket_points(b)).collect(),
            center: s.center,
            triples: s.triples,
            triples_to_end: super::TRIPLES_TO_END,
            kings: s.kings.clone(),
            trick: s.trick.clone(),
            led: s.led(),
            leader: s.leader,
            turn: s.turn,
            last_trick: s.last_trick.clone(),
            first_leader: s.first_leader,
            scores: s.scores.clone(),
            target: s.target(),
            round: s.round + 1,
            dealer: s.dealer,
            last_round: s.last_round.clone(),
            legal: me.map(|m| s.moves(m)).unwrap_or_default(),
        }
    }
}
