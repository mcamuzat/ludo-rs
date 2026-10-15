//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, Status, ANTE, DEFAULT_HANDS};
use crate::deck::Supply;
use crate::testing::ranks;

/// Hand 1 in play: these black stones and white stones, seat 0 to move,
/// everyone's ante in front. `deck` is drawn from its end.
pub fn table(blacks: &[u8], whites: &[u8], deck: &[u8]) -> State {
    let n = blacks.len();
    State {
        players: n as u8,
        hands_total: DEFAULT_HANDS,
        hand_no: 1,
        seed: 0,
        supply: Supply::from_parts(ranks(deck), Vec::new(), Vec::new(), 0, 0),
        whites: ranks(whites),
        blacks: blacks.to_vec(),
        status: vec![Status::In; n],
        min_take: 1,
        turn: Some(Seat(0)),
        phase: Phase::Play,
        pot: 0,
        chips: vec![-(ANTE as i32); n],
        events: Vec::new(),
    }
}
