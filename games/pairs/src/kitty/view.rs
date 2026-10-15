use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{rounds_total, Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand and what it passed and received, the
/// target, the discards and every hand size. Never another hand, another
/// seat's choice, nor the kitty's order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub rounds_total: u8,
    pub soft: bool,
    pub phase: Phase,
    pub hand_sizes: Vec<usize>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Pass or Bid: the card the viewer has chosen, not yet revealed.
    pub placed: Option<Rank>,
    /// This round: the card the viewer passed, and the one it received.
    pub passed: Option<Rank>,
    pub received: Option<Rank>,
    /// The target cards still to claim.
    pub target: Vec<Rank>,
    pub kitty_size: usize,
    /// Bids and unclaimed targets.
    pub discard: Vec<Rank>,
    /// Claim: who chooses.
    pub turn: Option<Seat>,
    pub scores: Vec<u32>,
    pub events: Vec<Event>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            round_no: s.round_no,
            rounds_total: rounds_total(s.soft),
            soft: s.soft,
            phase: s.phase,
            hand_sizes: s.hands.iter().map(Vec::len).collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()]),
            passed: me.and_then(|m| s.passed[m.index()]),
            received: me.and_then(|m| s.received[m.index()]),
            target: s.target.clone(),
            kitty_size: s.kitty.len(),
            discard: s.discard.clone(),
            turn: match s.phase {
                Phase::Claim => s.claims.first().copied(),
                _ => None,
            },
            scores: s.scores.clone(),
            events: s.events.clone(),
        }
    }
}
