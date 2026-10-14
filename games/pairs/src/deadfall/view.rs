use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State, ANTE};
use crate::deck::Rank;

/// What one seat sees: its own hand, every stack, hand sizes and the
/// balances. Never another hand, a door card another seat has chosen, nor
/// the cards left undealt. No statistics: reading the others is the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub hand_no: u8,
    pub hands_total: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Door: the card the viewer has chosen, not yet revealed.
    pub placed: Option<Rank>,
    pub turn: Option<Seat>,
    pub pot: u32,
    pub ante: u32,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    /// Face-up cards; the top is the last.
    pub stack: Vec<Rank>,
    pub hand_size: usize,
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
                    stack: s.stacks[i].clone(),
                    hand_size: s.hands[i].len(),
                    chips: s.chips[i],
                })
                .collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()]),
            turn: s.turn,
            pot: s.pot,
            ante: ANTE,
            events: s.events.clone(),
        }
    }
}
