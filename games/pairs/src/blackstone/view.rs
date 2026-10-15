use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State, Status, ANTE};
use crate::deck::Rank;

/// Everything is face up but the deck: every seat sees the same. The fold
/// cost is a rule, not an estimate. No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub hand_no: u8,
    pub hands_total: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The white stones dealt this hand.
    pub whites: Vec<Rank>,
    /// The fewest cards the player to move may take.
    pub min_take: u8,
    /// What folding costs now, ante included.
    pub fold_cost: u32,
    pub turn: Option<Seat>,
    pub pot: u32,
    pub ante: u32,
    pub deck: usize,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub blacks: u8,
    pub status: Status,
    /// Balance since the start of the game, an ante in front counted as
    /// spent.
    pub chips: i32,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        Self {
            hand_no: s.hand_no,
            hands_total: s.hands_total,
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    blacks: s.blacks[i],
                    status: s.status[i],
                    chips: s.chips[i],
                })
                .collect(),
            whites: s.whites.clone(),
            min_take: s.min_take,
            fold_cost: ANTE + s.fold_cost(),
            turn: s.turn,
            pot: s.pot,
            ante: ANTE,
            deck: s.supply.deck_len(),
            events: s.events.clone(),
        }
    }
}
