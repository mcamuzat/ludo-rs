//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State};
use crate::testing::ranks;

/// Round 1, the last seat dealt, seat 0 to play: these hands, this Cave
/// and this deck (`deck[0]` drawn first). Nothing captured, no score yet.
pub fn table(hands: &[&[u8]], cave: &[u8], deck: &[u8]) -> State {
    let n = hands.len();
    let mut deck = ranks(deck);
    deck.reverse();
    State {
        players: n as u8,
        round_no: 1,
        seed: 0,
        dealer: Seat(n as u8 - 1),
        deck,
        cave: ranks(cave),
        hands: hands.iter().map(|h| ranks(h)).collect(),
        captured: vec![Vec::new(); n],
        scores: vec![0; n],
        phase: Phase::Play,
        turn: Seat(0),
        events: Vec::new(),
    }
}
