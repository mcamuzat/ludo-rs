use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its hand, the Cave, everyone's captures and the
/// scores. Never another hand nor the deck's order. No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    /// Rounds in the game: one per player.
    pub rounds: u8,
    pub phase: Phase,
    pub dealer: Seat,
    pub turn: Option<Seat>,
    pub cave: Vec<Rank>,
    pub deck_size: usize,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    pub seats: Vec<SeatView>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    /// Captured this round, sorted.
    pub captured: Vec<Rank>,
    pub hand_size: usize,
    pub score: u16,
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
            rounds: s.players,
            phase: s.phase,
            dealer: s.dealer,
            turn: playing.then_some(s.turn),
            cave: s.cave.clone(),
            deck_size: s.deck.len(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            seats: (0..usize::from(s.players))
                .map(|i| {
                    let mut captured = s.captured[i].clone();
                    captured.sort_unstable();
                    SeatView {
                        captured,
                        hand_size: s.hands[i].len(),
                        score: s.scores[i],
                    }
                })
                .collect(),
            events: s.events.clone(),
        }
    }
}
