//! Hand-built tables for tests.

use ludo_core::Seat;

use super::cards::Card;
use super::{Config, Phase, State};

pub fn card(value: i8) -> Card {
    Card::new(value).expect("−5 or a rank from 1 to 10")
}

pub fn c(values: &[i8]) -> Vec<Card> {
    values.iter().map(|&v| card(v)).collect()
}

/// Hand 1 of 6, the last seat dealt, seat 0 to play: these hands, this
/// line (oldest first) and this deck (`deck[0]` drawn first). Nothing
/// picked up, no discard, no score yet.
pub fn table(hands: &[&[i8]], line: &[i8], deck: &[i8]) -> State {
    let n = hands.len();
    let mut deck = c(deck);
    deck.reverse();
    State {
        players: n as u8,
        config: Config::default(),
        round_no: 1,
        seed: 0,
        dealer: Seat(n as u8 - 1),
        deck,
        discard: Vec::new(),
        line: c(line),
        hands: hands.iter().map(|h| c(h)).collect(),
        taken: vec![Vec::new(); n],
        scores: vec![0; n],
        out: None,
        phase: Phase::Play,
        turn: Seat(0),
        events: Vec::new(),
    }
}
