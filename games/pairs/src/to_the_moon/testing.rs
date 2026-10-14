//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, DEFAULT_ROUNDS};
use crate::deck::Supply;
use crate::testing::ranks;

/// Round 1, seat 0 deals and has led `trick`; seat 1 to move (whatever
/// the number of players). `draws` are the next cards of the deck, in
/// order.
pub fn table(hands: &[&[u8]], trick: &[u8], draws: &[u8]) -> State {
    let n = hands.len();
    let mut deck = ranks(draws);
    deck.reverse();
    State {
        players: n as u8,
        rounds_total: DEFAULT_ROUNDS,
        round_no: 1,
        dealer: Seat(0),
        supply: Supply::from_parts(deck, Vec::new(), Vec::new(), 0, 0),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        known: vec![Vec::new(); n],
        trick: ranks(trick),
        leader: Seat(0),
        out: vec![false; n],
        turn: Some(Seat(1 % n as u8)),
        phase: Phase::Play,
        scores: vec![0; n],
        events: Vec::new(),
    }
}
