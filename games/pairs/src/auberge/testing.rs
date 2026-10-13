//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, DEFAULT_ROUNDS};
use crate::deck::Supply;
use crate::testing::ranks;

fn supply(draws: &[u8]) -> Supply {
    let mut deck = ranks(draws);
    deck.reverse();
    Supply::from_parts(deck, Vec::new(), Vec::new(), 0, 0)
}

fn blank(hands: &[&[u8]], draws: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        rounds: DEFAULT_ROUNDS,
        round: 1,
        supply: supply(draws),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        fronts: vec![Vec::new(); n],
        placed: vec![None; n],
        out: vec![false; n],
        known: vec![Vec::new(); n],
        phase: Phase::Setup,
        turn: None,
        wins: vec![0; n],
        events: Vec::new(),
    }
}

/// Round 1 before anyone has placed a card; `draws` are the next cards of
/// the draw pile, in order.
pub fn setup(hands: &[&[u8]], draws: &[u8]) -> State {
    blank(hands, draws)
}

/// Round 1 in play: these hands and face-up cards, seat 0 to move.
pub fn table(hands: &[&[u8]], fronts: &[&[u8]], draws: &[u8]) -> State {
    let mut s = blank(hands, draws);
    s.fronts = fronts.iter().map(|f| ranks(f)).collect();
    s.phase = Phase::Play;
    s.turn = Some(Seat(0));
    s
}
