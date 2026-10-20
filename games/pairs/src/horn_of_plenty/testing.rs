//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1, seat 0 to play: these face-down `hands` and face-up `shown`
/// hands, `undealt` in the draw pile, no score yet.
pub fn table(hands: &[&[u8]], shown: &[&[u8]], undealt: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        round_no: 1,
        supply: Supply::from_parts(ranks(undealt), Vec::new(), Vec::new(), 0, 0),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        shown: shown.iter().map(|h| ranks(h)).collect(),
        scores: vec![0; n],
        phase: Phase::Declare,
        turn: Seat(0),
        table: None,
        chooser: None,
        recipients: Vec::new(),
        events: Vec::new(),
    }
}
