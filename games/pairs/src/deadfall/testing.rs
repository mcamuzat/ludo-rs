//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, ANTE, DEFAULT_HANDS};
use crate::deck::Supply;
use crate::testing::ranks;

/// Hand 1 before anyone has chosen a door card: everyone has paid the ante.
/// `undealt` stays in the draw pile.
pub fn door(hands: &[&[u8]], undealt: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        hands_total: DEFAULT_HANDS,
        hand_no: 1,
        supply: Supply::from_parts(ranks(undealt), Vec::new(), Vec::new(), 0, 0),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        stacks: vec![Vec::new(); n],
        stamps: vec![0; n],
        clock: 0,
        placed: vec![None; n],
        phase: Phase::Door,
        turn: None,
        pot: ANTE * n as u32,
        chips: vec![-(ANTE as i32); n],
        events: Vec::new(),
    }
}

/// Hand 1 in play: these hands and stacks, the top cards played in seat
/// order, seat 0 to move.
pub fn table(hands: &[&[u8]], stacks: &[&[u8]], undealt: &[u8]) -> State {
    let mut s = door(hands, undealt);
    s.stacks = stacks.iter().map(|stack| ranks(stack)).collect();
    s.stamps = (1..=stacks.len() as u32).collect();
    s.clock = stacks.len() as u32;
    s.phase = Phase::Play;
    s.turn = Some(Seat(0));
    s
}
