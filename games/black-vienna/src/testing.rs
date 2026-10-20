//! Hand-built tables for tests.

use ludo_core::Seat;

use crate::material::{CardId, Suspect, INVESTIGATIONS};
use crate::{Phase, State, TOKENS};

/// Suspects by their letters ("ACL" → A, C, L), sorted.
pub fn letters(text: &str) -> Vec<Suspect> {
    let mut out: Vec<Suspect> = text
        .chars()
        .map(|c| Suspect::from_letter(c).unwrap_or_else(|| panic!("lettre inconnue : {c}")))
        .collect();
    out.sort();
    out
}

/// A trio by its letters, sorted.
pub fn trio(text: &str) -> [Suspect; 3] {
    let v = letters(text);
    [v[0], v[1], v[2]]
}

/// An investigation card by its name ("BLM").
pub fn card(name: &str) -> CardId {
    let at = INVESTIGATIONS
        .iter()
        .position(|&c| c == name)
        .unwrap_or_else(|| panic!("carte inconnue : {name}"));
    CardId::new(at as u8).expect("36 cards")
}

/// Seat 0 deals and plays first: these hands, this trio, and the piles as
/// card names separated by spaces, the top last. Full reserve, nobody
/// asked yet.
pub fn table(hands: &[&str], trio_letters: &str, piles: [&str; 3]) -> State {
    let n = hands.len();
    let pile = |text: &str| -> Vec<CardId> { text.split_whitespace().map(card).collect() };
    State {
        players: n as u8,
        dealer: Seat(0),
        seed: 0,
        hands: hands.iter().map(|h| letters(h)).collect(),
        trio: trio(trio_letters),
        piles: [pile(piles[0]), pile(piles[1]), pile(piles[2])],
        answers: Vec::new(),
        reserve: TOKENS,
        asked: vec![false; n],
        wrong: vec![false; n],
        phase: Phase::Turn,
        turn: Seat(0),
        last: None,
        final_asked: false,
        guesses: vec![None; n],
        accuser: None,
        scores: vec![0; n],
        events: Vec::new(),
    }
}
