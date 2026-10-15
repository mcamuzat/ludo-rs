//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, Status};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1 of `rounds`, every player in, seat 0 to act. `deck` is drawn
/// from its end; nothing to reshuffle.
pub fn table(stacks: &[&[u8]], dealer: &[u8], rounds: u8, deck: &[u8]) -> State {
    let n = stacks.len();
    State {
        players: n as u8,
        rounds_total: rounds,
        round_no: 1,
        supply: Supply::from_parts(ranks(deck), Vec::new(), Vec::new(), 0, 0),
        stacks: stacks.iter().map(|s| ranks(s)).collect(),
        dealer: ranks(dealer),
        status: vec![Status::In; n],
        turn: Some(Seat(0)),
        phase: Phase::Play,
        chips: vec![0; n],
        events: Vec::new(),
    }
}
