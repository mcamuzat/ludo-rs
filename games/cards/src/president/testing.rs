//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Config, Phase, State, Title};
use crate::deck::{Card, Rank, Suit};

/// "7h", "Ts", "Qd", "2c"…, and "Jo" for a joker.
pub fn c(text: &str) -> Card {
    if text == "Jo" {
        return Card::Joker;
    }
    let mut chars = text.chars();
    let rank = match chars.next() {
        Some('2') => Rank::Two,
        Some('3') => Rank::Three,
        Some('4') => Rank::Four,
        Some('5') => Rank::Five,
        Some('6') => Rank::Six,
        Some('7') => Rank::Seven,
        Some('8') => Rank::Eight,
        Some('9') => Rank::Nine,
        Some('T') => Rank::Ten,
        Some('J') => Rank::Jack,
        Some('Q') => Rank::Queen,
        Some('K') => Rank::King,
        Some('A') => Rank::Ace,
        other => panic!("rang inconnu {other:?} dans {text}"),
    };
    let suit = match chars.next() {
        Some('s') => Suit::Spades,
        Some('h') => Suit::Hearts,
        Some('d') => Suit::Diamonds,
        Some('c') => Suit::Clubs,
        other => panic!("couleur inconnue {other:?} dans {text}"),
    };
    Card::Natural { rank, suit }
}

pub fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace().map(c).collect()
}

/// Round 1, in play: these hands, `turn` to move, no trick on the table.
pub fn table(config: Config, hands: &[&str], turn: u8) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        config,
        seed: 0,
        round_no: 1,
        phase: Phase::Play,
        hands: hands.iter().map(|h| cards(h)).collect(),
        set_aside: Vec::new(),
        titles: Vec::new(),
        reversed: false,
        trick: None,
        passes: Vec::new(),
        turn: Some(Seat(turn)),
        out: Vec::new(),
        fouled: Vec::new(),
        transfers: Vec::new(),
        to_give: Vec::new(),
        scores: vec![0; n],
        events: Vec::new(),
    }
}

/// Round 2, after a round that gave these titles: these hands are dealt
/// and the exchange begins.
pub fn next_round(config: Config, titles: &[Title], hands: &[&str]) -> State {
    let mut s = table(config, &vec![""; titles.len()], 0);
    s.titles = titles.to_vec();
    s.begin_round(hands.iter().map(|h| cards(h)).collect(), Vec::new());
    s
}
