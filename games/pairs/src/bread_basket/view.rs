use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::cards::{filling_points, Card};
use super::{Event, Phase, State};

/// What one seat sees: its own hand, the line, the discards, what everyone
/// picked up, and the scores. Never another hand nor the deck's order. No
/// statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    /// Hands in the game.
    pub rounds: u8,
    pub phase: Phase,
    pub dealer: Seat,
    pub turn: Option<Seat>,
    /// Oldest card first.
    pub line: Vec<Card>,
    pub deck_size: usize,
    pub discard: Vec<Card>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Card>,
    pub seats: Vec<SeatView>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    /// The middle cards picked up this hand.
    pub taken: Vec<Card>,
    pub taken_points: i32,
    pub hand_size: usize,
    pub score: i32,
    /// Ran out of cards first this hand.
    pub out: bool,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let playing = s.phase == Phase::Play;
        Self {
            round_no: s.round_no,
            rounds: s.config.rounds.count(s.players),
            phase: s.phase,
            dealer: s.dealer,
            turn: playing.then_some(s.turn),
            line: s.line.clone(),
            deck_size: s.deck.len(),
            discard: s.discard.clone(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    taken: s.taken[i].clone(),
                    taken_points: filling_points(&s.taken[i]),
                    hand_size: s.hands[i].len(),
                    score: s.scores[i],
                    out: s.out == Some(Seat(i as u8)),
                })
                .collect(),
            events: s.events.clone(),
        }
    }
}
