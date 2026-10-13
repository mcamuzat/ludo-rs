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
        supply: supply(draws),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        houses: vec![Vec::new(); n],
        street: Vec::new(),
        placed: vec![None; n],
        out: vec![false; n],
        phase: Phase::Setup,
        turn: None,
        pot: ANTE * n as u32,
        chips: vec![-(ANTE as i32); n],
        events: Vec::new(),
    }
}

/// Hand 1 in play: these hands, houses and street, seat 0 to move.
pub fn table(hands: &[&[u8]], houses: &[&[u8]], street: &[u8], draws: &[u8]) -> State {
    let mut s = setup(hands, draws);
    s.houses = houses.iter().map(|h| ranks(h)).collect();
    s.street = ranks(street);
    s.phase = Phase::Play;
    s.turn = Some(Seat(0));
    s
}
