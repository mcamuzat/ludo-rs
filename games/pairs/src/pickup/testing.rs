//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Config, Phase, State};
use crate::testing::ranks;

/// Deal 1 (one-deal game), the last seat dealt, everyone to choose: these
/// hands, this pool and this deck (`deck[0]` drawn first). Empty piles, no
/// score yet.
pub fn table(hands: &[&[u8]], pool: &[u8], deck: &[u8]) -> State {
    let n = hands.len();
    let mut deck = ranks(deck);
    deck.reverse();
    State {
        players: n as u8,
        config: Config::default(),
        deal_no: 1,
        seed: 0,
        dealer: Seat(n as u8 - 1),
        deck,
        pool: ranks(pool),
        discard: Vec::new(),
        hands: hands.iter().map(|h| ranks(h)).collect(),
        chosen: vec![None; n],
        revealed: vec![None; n],
        waiting: Vec::new(),
        taking: None,
        last_actor: Seat(n as u8 - 1),
        piles: vec![Vec::new(); n],
        scores: vec![0; n],
        phase: Phase::Choose,
        events: Vec::new(),
    }
}
