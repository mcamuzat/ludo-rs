//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, Status, ANTE};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1 of `rounds`, everyone anted and undecided, seat 0 on the
/// button and to act. `deck` is drawn from its end.
pub fn table(players: u8, rounds: u8, center: &[u8], deck: &[u8]) -> State {
    let n = usize::from(players);
    State {
        players,
        rounds_total: rounds,
        round_no: 1,
        seed: 0,
        supply: Supply::from_parts(ranks(deck), Vec::new(), Vec::new(), 0, 0),
        center: ranks(center),
        status: vec![Status::In; n],
        button: Seat(0),
        turn: Some(Seat(0)),
        phase: Phase::Play,
        pot: ANTE * u32::from(players),
        chips: vec![-(ANTE as i32); n],
        events: Vec::new(),
    }
}
