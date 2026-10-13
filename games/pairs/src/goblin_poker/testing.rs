//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, ANTE, DEFAULT_HANDS};
use crate::deck::Supply;
use crate::testing::ranks;

fn supply(draws: &[u8]) -> Supply {
    let mut deck = ranks(draws);
    deck.reverse();
    Supply::from_parts(deck, Vec::new(), Vec::new(), 0, 0)
}

/// Hand 1 before anyone has placed a card: everyone has paid the ante.
/// `draws` are the next cards of the draw pile, in order.
pub fn setup(hands: &[&[u8]], draws: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        hands_total: DEFAULT_HANDS,
        hand_no: 1,
        round: 0,
        supply: supply(draws),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        stacks: vec![Vec::new(); n],
        placed: vec![None; n],
        pitched: vec![None; n],
        out: vec![false; n],
        phase: Phase::Setup,
        turn: None,
        pot: ANTE * n as u32,
        chips: vec![-(ANTE as i32); n],
        events: Vec::new(),
    }
}

/// Hand 1, round 1: these hands and stacks, nobody has acted, seat 0 to
/// move (whatever the stacks say).
pub fn table(hands: &[&[u8]], stacks: &[&[u8]], draws: &[u8]) -> State {
    let mut s = setup(hands, draws);
    s.stacks = stacks.iter().map(|stack| ranks(stack)).collect();
    s.phase = Phase::Play;
    s.round = 1;
    s.turn = Some(Seat(0));
    s
}
