use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// Everything is face up but the draw pile: both players see the same.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub target_score: u16,
    pub phase: Phase,
    pub columns: Vec<ColumnView>,
    /// The card the player to move has drawn.
    pub drawn: Option<Rank>,
    pub turn: Option<Seat>,
    pub scores: Vec<u32>,
    pub deck_size: usize,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnView {
    pub target: Rank,
    /// Each seat's side of the column, in the order played.
    pub sides: Vec<Vec<Rank>>,
    /// Points each seat has played there.
    pub points: Vec<u32>,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        Self {
            round_no: s.round_no,
            target_score: s.target_score,
            phase: s.phase,
            columns: s
                .targets
                .iter()
                .enumerate()
                .map(|(c, &target)| ColumnView {
                    target,
                    sides: s.played.iter().map(|side| side[c].clone()).collect(),
                    points: (0..2).map(|i| s.points(Seat(i), c)).collect(),
                })
                .collect(),
            drawn: s.drawn,
            turn: s.turn,
            scores: s.scores.clone(),
            deck_size: s.supply.deck_len(),
            events: s.events.clone(),
        }
    }
}
