//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, Status, DEFAULT_TARGET};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1 in play: these stacks, all playing, seat 0 to move. `deck` is
/// drawn from its end; nothing to reshuffle.
pub fn table(stacks: &[&[u8]], deck: &[u8]) -> State {
    let n = stacks.len();
    State {
        players: n as u8,
        target: DEFAULT_TARGET,
        round_no: 1,
        supply: Supply::from_parts(ranks(deck), Vec::new(), Vec::new(), 0, 0),
        stacks: stacks.iter().map(|s| ranks(s)).collect(),
        status: vec![Status::Playing; n],
        turn: Some(Seat(0)),
        phase: Phase::Play,
        scores: vec![0; n],
        events: Vec::new(),
    }
}
