use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State, Status};
use crate::deck::Rank;

/// Everything is face up but the draw pile: every seat sees the same.
/// No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub target: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    pub turn: Option<Seat>,
    pub deck: usize,
    pub discard: usize,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub stack: Vec<Rank>,
    /// The stack's card points.
    pub total: u32,
    pub status: Status,
    /// Game points.
    pub score: u32,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        Self {
            round_no: s.round_no,
            target: s.target,
            phase: s.phase,
            seats: (0..s.players)
                .map(Seat)
                .map(|seat| SeatView {
                    stack: s.stacks[seat.index()].clone(),
                    total: s.total(seat),
                    status: s.status[seat.index()],
                    score: s.scores[seat.index()],
                })
                .collect(),
            turn: s.turn,
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            events: s.events.clone(),
        }
    }
}
