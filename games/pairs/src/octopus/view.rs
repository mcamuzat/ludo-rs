use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand and what it passed and received, the
/// cards played, hand sizes and the balances. Never another hand, another
/// seat's choice, nor the cards left undealt. No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub hand_no: u8,
    pub hands_total: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Play or Pass: the card the viewer has chosen, not yet revealed.
    pub placed: Option<Rank>,
    /// Since the last pass: the card the viewer passed, and received.
    pub passed: Option<Rank>,
    pub received: Option<Rank>,
    /// The cards played this hand.
    pub played: Vec<Rank>,
    /// Declare: whose turn; Call: who calls.
    pub turn: Option<Seat>,
    pub leader: Seat,
    pub pot: u32,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub hand_size: usize,
    pub folded: bool,
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
                    hand_size: s.hands.get(i).map_or(0, Vec::len),
                    folded: s.folded[i],
                    chips: s.chips[i],
                })
                .collect(),
            hand: me
                .and_then(|m| s.hands.get(m.index()).cloned())
                .unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()]),
            passed: me.and_then(|m| s.passed[m.index()]),
            received: me.and_then(|m| s.received[m.index()]),
            played: s.played.clone(),
            turn: match s.phase {
                Phase::Declare => s.turn,
                Phase::Call => s.caller,
                _ => None,
            },
            leader: s.leader,
            pot: s.pot,
            events: s.events.clone(),
        }
    }
}
