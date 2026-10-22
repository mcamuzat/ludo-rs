use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{hands_in_game, Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its hand, the round in progress, the cards of
/// earlier rounds, the scores. Never another hand nor the undealt cards.
/// No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub hand_no: u8,
    pub hands: u8,
    pub phase: Phase,
    pub dealer: Seat,
    pub leader: Seat,
    pub turn: Option<Seat>,
    /// The round in progress: who played what, in order.
    pub trick: Vec<(Seat, Rank)>,
    pub discard: Vec<Rank>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    pub seats: Vec<SeatView>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub hand_size: usize,
    pub score: u16,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            hand_no: s.hand_no,
            hands: hands_in_game(s.players),
            phase: s.phase,
            dealer: s.dealer,
            leader: s.leader,
            turn: (s.phase != Phase::Over).then_some(s.turn),
            trick: s.trick.clone(),
            discard: s.discard.clone(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    hand_size: s.hands[i].len(),
                    score: s.scores[i],
                })
                .collect(),
            events: s.events.clone(),
        }
    }
}
