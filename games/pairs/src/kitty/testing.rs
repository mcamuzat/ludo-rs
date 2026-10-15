//! Hand-built tables for tests.

use super::{is_soft, Event, Phase, State};
use crate::testing::ranks;

/// Round `round` in the Pass phase: these hands, this target, `kitty`
/// left to turn (the next target last).
pub fn table(hands: &[&[u8]], round: u8, target: &[u8], kitty: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        soft: is_soft(n as u8),
        round_no: round,
        kitty: ranks(kitty),
        removed: Vec::new(),
        hands: hands.iter().map(|h| ranks(h)).collect(),
        target: ranks(target),
        placed: vec![None; n],
        passed: vec![None; n],
        received: vec![None; n],
        bids: Vec::new(),
        claims: Vec::new(),
        discard: Vec::new(),
        phase: Phase::Pass,
        scores: Vec::new(),
        events: vec![Event::TargetRevealed {
            round,
            cards: ranks(target),
        }],
    }
}
