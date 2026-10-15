//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1 of `rounds_total` = 1 (unless changed), in play: these hands,
/// these targets, `leader` to shoot first. `deck` is drawn from its end.
pub fn table(hands: &[&[u8]], targets: &[u8], leader: u8, deck: &[u8]) -> State {
    let n = hands.len();
    let targets = ranks(targets);
    State {
        players: n as u8,
        rounds_total: 1,
        round_no: 1,
        supply: Supply::from_parts(ranks(deck), Vec::new(), Vec::new(), 0, 0),
        hands: hands.iter().map(|h| ranks(h)).collect(),
        known: vec![Vec::new(); n],
        top: targets.iter().copied().max(),
        targets,
        shooters: Vec::new(),
        lowest_shot: None,
        groups: Vec::new(),
        tie: None,
        leader: Seat(leader),
        last_taker: None,
        turn: Some(Seat(leader)),
        phase: Phase::Play,
        discard: Vec::new(),
        scores: Vec::new(),
        events: Vec::new(),
    }
}
