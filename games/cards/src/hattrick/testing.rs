//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Card, Color, Config, State};

/// A card from its short name: `r7`, `b20`, `g1` (red, blue, green).
pub fn c(name: &str) -> Card {
    let (color, value) = name.split_at(1);
    let color = match color {
        "r" => Color::Red,
        "b" => Color::Blue,
        "g" => Color::Green,
        other => panic!("unknown colour {other}"),
    };
    Card {
        color,
        value: value.parse().expect("a value"),
    }
}

pub fn cards(names: &str) -> Vec<Card> {
    names.split_whitespace().map(c).collect()
}

/// The first round, in play: these hands, `leader` to lead (the dealer sits
/// at its right).
pub fn table(hands: &[&str], leader: u8) -> State {
    let n = hands.len();
    State {
        config: Config::default(),
        players: n as u8,
        seed: 0,
        round: 0,
        dealer: Seat((leader + n as u8 - 1) % n as u8),
        hands: hands.iter().map(|h| cards(h)).collect(),
        trick: Vec::new(),
        last_trick: None,
        turn: Some(Seat(leader)),
        won: vec![[0; 3]; n],
        passed: vec![0; n],
        scores: vec![0; n],
        last_round: None,
    }
}
