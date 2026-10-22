//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State};
use crate::testing::ranks;

/// Hand 1, the last seat dealt, seat 0 leading the first round: these
/// hands, nothing undealt nor played, no score yet.
pub fn table(hands: &[&[u8]]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        hand_no: 1,
        seed: 0,
        dealer: Seat(n as u8 - 1),
        undealt: Vec::new(),
        hands: hands.iter().map(|h| ranks(h)).collect(),
        trick: Vec::new(),
        discard: Vec::new(),
        leader: Seat(0),
        scores: vec![0; n],
        phase: Phase::Play,
        turn: Seat(0),
        events: Vec::new(),
    }
}
