use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own stack in order, every face-up card, stack
/// sizes, the pot and the balances. Never another stack, nor the deck.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub hand_no: u8,
    pub hands_total: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's face-down stack, top card first (empty for a spectator).
    pub stack: Vec<Rank>,
    /// Arrange: the order the viewer has chosen, top first.
    pub placed: Option<Vec<Rank>>,
    pub turn: Option<Seat>,
    pub pot: u32,
    pub deck: usize,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub faceup: Vec<Rank>,
    pub stack_size: usize,
    pub out: bool,
    /// Balance since the start of the game.
    pub chips: i32,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            hand_no: s.hand_no,
            hands_total: s.hands_total,
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    faceup: s.faceup[i].clone(),
                    stack_size: s.stacks[i].len(),
                    out: s.out[i],
                    chips: s.chips[i],
                })
                .collect(),
            stack: me
                .map(|m| s.stacks[m.index()].iter().rev().copied().collect())
                .unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()].clone()),
            turn: s.turn,
            pot: s.pot,
            deck: s.supply.deck_len(),
            events: s.events.clone(),
        }
    }
}
