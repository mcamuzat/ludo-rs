//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1, seat 0 to cook: these hands and recipes, an empty Judge (set
/// `judge` to stage a contest), nothing undealt, no score yet.
pub fn table(hands: &[&[u8]], recipes: &[&[u8]]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        round_no: 1,
        supply: Supply::from_parts(Vec::new(), Vec::new(), Vec::new(), 0, 0),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        recipes: recipes.iter().map(|h| ranks(h)).collect(),
        passed: vec![false; n],
        judge: Vec::new(),
        scores: vec![0; n],
        leader: Seat(0),
        phase: Phase::Cook,
        turn: Seat(0),
        events: Vec::new(),
    }
}
