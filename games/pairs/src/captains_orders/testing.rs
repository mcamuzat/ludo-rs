//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, DEFAULT_WINS};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1 with these hands, seat 0 to move with no Order out (seat
/// `n - 1` dealt). `undealt` stays in the draw pile.
pub fn table(hands: &[&[u8]], undealt: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        wins_needed: DEFAULT_WINS,
        round: 1,
        dealer: Seat(n as u8 - 1),
        wins: vec![0; n],
        supply: Supply::from_parts(ranks(undealt), Vec::new(), Vec::new(), 0, 0),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        discarded: vec![Vec::new(); n],
        order: None,
        phase: Phase::Turn,
        turn: Some(Seat(0)),
        log: Vec::new(),
        last_round: None,
    }
}
