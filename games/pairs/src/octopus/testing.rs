//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, DEFAULT_HANDS, OPEN_ANTE};
use crate::testing::ranks;

/// Hand 1, everyone has stayed in and anted 2: these hands, in the Play
/// phase. `undealt` holds the rest.
pub fn table(hands: &[&[u8]], undealt: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        hands_total: DEFAULT_HANDS,
        hand_no: 1,
        seed: 0,
        undealt: ranks(undealt),
        hands: hands.iter().map(|h| ranks(h)).collect(),
        folded: vec![false; n],
        leader: Seat(0),
        turn: None,
        placed: vec![None; n],
        passed: vec![None; n],
        received: vec![None; n],
        tracked: vec![None; n],
        played: Vec::new(),
        caller: None,
        phase: Phase::Play,
        pot: OPEN_ANTE * n as u32,
        chips: vec![-(OPEN_ANTE as i32); n],
        losers: None,
        events: Vec::new(),
    }
}
