use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand, the countdown, hand sizes, the cards
/// of each hand everyone has seen go in, who is out and the scores. Never
/// another hand's other cards nor the draw pile. No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub rounds_total: u8,
    pub phase: Phase,
    pub dealer: Seat,
    pub leader: Seat,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// The countdown, in play order.
    pub trick: Vec<Rank>,
    pub turn: Option<Seat>,
    pub deck: usize,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub hand_size: usize,
    /// Cards of this hand everyone has seen go in.
    pub known: Vec<Rank>,
    /// Out of the current trick.
    pub out: bool,
    pub score: u32,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            round_no: s.round_no,
            rounds_total: s.rounds_total,
            phase: s.phase,
            dealer: s.dealer,
            leader: s.leader,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    hand_size: s.hands[i].len(),
                    known: s.known[i].clone(),
                    out: s.out[i],
                    score: s.scores[i],
                })
                .collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            trick: s.trick.clone(),
            turn: s.turn,
            deck: s.supply.deck_len(),
            events: s.events.clone(),
        }
    }
}
