//! Hand-built tables for tests.

use super::{Config, Phase, State};
pub use crate::president::testing::{c, cards};

/// Competition 1 of a one-competition game, turn 1, everyone to choose:
/// these hands, this pool and this deck (its first card drawn first). No
/// card seen taken, no score yet.
pub fn table(hands: &[&str], pool: &str, deck: &str) -> State {
    let n = hands.len();
    let mut deck = cards(deck);
    deck.reverse();
    State {
        players: n as u8,
        config: Config { competitions: 1 },
        competition: 1,
        turn_no: 1,
        seed: 0,
        deck,
        pool: cards(pool),
        discard: Vec::new(),
        hands: hands.iter().map(|h| cards(h)).collect(),
        known: vec![Vec::new(); n],
        chosen: vec![None; n],
        revealed: vec![None; n],
        waiting: Vec::new(),
        picking: None,
        scores: vec![0; n],
        last_showdown: None,
        phase: Phase::Choose,
        events: Vec::new(),
    }
}
