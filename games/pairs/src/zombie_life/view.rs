use ludo_core::Seat;
use serde::Serialize;

use super::{Event, Phase, State, Status};
use crate::deck::Rank;

/// What everyone sees: the refuges, the horde, the scores, the card just
/// drawn and the dealer's cards. Only the draw pile is hidden. No
/// statistics: weighing a draw is the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round: u16,
    pub target: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    pub horde: Vec<Rank>,
    /// Seat to move; the dealer while dealing.
    pub turn: Option<Seat>,
    /// Placing: the card `turn` has drawn.
    pub drawn: Option<Rank>,
    /// Dealing: the dealer's cards not yet given.
    pub dealing: Vec<Rank>,
    /// Dealing: the seat the next card goes to.
    pub deal_to: Option<Seat>,
    pub deck: usize,
    pub discard: usize,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub face_up: Vec<Rank>,
    /// Cards turned face down by passing: they score nothing.
    pub face_down: u8,
    pub status: Status,
    pub score: u16,
}

impl View {
    pub fn of(s: &State) -> Self {
        Self {
            round: s.round,
            target: s.target,
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| {
                    let refuge = &s.refuges[i];
                    let face_down = s.hidden[i].min(refuge.len() as u8);
                    SeatView {
                        face_up: refuge[..refuge.len() - usize::from(face_down)].to_vec(),
                        face_down,
                        status: s.status[i],
                        score: s.scores[i],
                    }
                })
                .collect(),
            horde: s.horde.clone(),
            turn: s.turn,
            drawn: s.drawn,
            dealing: s.dealing.clone(),
            deal_to: s.deal_to(),
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            events: s.events.clone(),
        }
    }
}
