//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, ANTE, DEFAULT_HANDS};
use crate::deck::Supply;
use crate::testing::ranks;

/// Hand 1 in play: these face-down stacks (top card first) and face-up
/// cards, seat 0 to play. `deck` is drawn from its end.
pub fn table(stacks: &[&[u8]], faceup: &[&[u8]], deck: &[u8]) -> State {
    let n = stacks.len();
    State {
        players: n as u8,
        hands_total: DEFAULT_HANDS,
        hand_no: 1,
        seed: 0,
        supply: Supply::from_parts(ranks(deck), Vec::new(), Vec::new(), 0, 0),
        stacks: stacks
            .iter()
            .map(|st| ranks(st).into_iter().rev().collect())
            .collect(),
        placed: vec![None; n],
        faceup: faceup.iter().map(|f| ranks(f)).collect(),
        out: vec![false; n],
        first: Seat(0),
        turn: Some(Seat(0)),
        phase: Phase::Play,
        pot: ANTE * n as u32,
        chips: vec![-(ANTE as i32); n],
        events: Vec::new(),
    }
}
