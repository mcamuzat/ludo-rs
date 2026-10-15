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
    pub stacks: Vec<Vec<Rank>>,
    pub status: Vec<Status>,
    pub dealer: Vec<Rank>,
    pub turn: Option<Seat>,
    /// Each player's balance against the house.
    pub chips: Vec<i32>,
    /// What folding would cost the player to move.
    pub fold_cost: Option<u32>,
    pub deck: usize,
    pub events: Vec<Event>,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        Self {
            round_no: s.round_no,
            rounds_total: s.rounds_total,
            phase: s.phase,
            stacks: s.stacks.clone(),
            status: s.status.clone(),
            dealer: s.dealer.clone(),
            turn: s.turn,
            chips: s.chips.clone(),
            fold_cost: s.turn.map(|t| s.fold_cost(t)),
            deck: s.supply.deck_len(),
            events: s.events.clone(),
        }
    }
}
