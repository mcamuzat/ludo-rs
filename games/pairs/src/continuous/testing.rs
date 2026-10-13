//! Hand-built tables for tests (of this crate and of its users).

use ludo_core::Seat;

use super::{default_target, State, BURN};
use crate::deck::{full_deck, Supply};
pub use crate::testing::{r, ranks};

/// A supply whose next draws are `draws`, in that order; empty discard.
pub fn supply(draws: &[u8]) -> Supply {
    let mut deck = ranks(draws);
    deck.reverse();
    Supply::from_parts(deck, Vec::new(), Vec::new(), BURN, 0)
}

/// A game in progress: these stacks, seat 0 to move, no score yet, and
/// `draws` as the next cards of the draw pile.
pub fn table(stacks: &[&[u8]], draws: &[u8]) -> State {
    let players = stacks.len() as u8;
    State {
        players,
        target: default_target(players),
        supply: supply(draws),
        stacks: stacks.iter().map(|s| ranks(s)).collect(),
        scored: vec![Vec::new(); stacks.len()],
        turn: Some(Seat(0)),
        loser: None,
        events: Vec::new(),
    }
}

/// Like `table`, with every other card of the deck in the draw pile.
pub fn table_with_rest(stacks: &[&[u8]]) -> State {
    let mut rest = full_deck();
    for card in stacks.iter().flat_map(|s| ranks(s)) {
        let at = rest
            .iter()
            .position(|&c| c == card)
            .expect("more cards of a rank than the deck holds");
        rest.remove(at);
    }
    let draws: Vec<u8> = rest.iter().map(|c| c.value()).collect();
    table(stacks, &draws)
}
