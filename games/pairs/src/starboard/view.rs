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
    pub rounds_total: u8,
    pub phase: Phase,
    pub center: Vec<Rank>,
    pub seats: Vec<SeatView>,
    pub button: Seat,
    pub turn: Option<Seat>,
    pub pot: u32,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub status: Status,
    /// Balance since the start of the game.
    pub chips: i32,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        Self {
            round_no: s.round_no,
            rounds_total: s.rounds_total,
            phase: s.phase,
            center: s.center.clone(),
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    status: s.status[i],
                    chips: s.chips[i],
                })
                .collect(),
            button: s.button,
            turn: s.turn,
            pot: s.pot,
            events: s.events.clone(),
        }
    }
}
