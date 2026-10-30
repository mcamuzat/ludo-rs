//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Card, Color, Config, State};

/// A card from its short name: `r7`, `y12`, `g1`, `b18`, `v3`.
pub fn c(name: &str) -> Card {
    let (color, value) = name.split_at(1);
    let color = match color {
        "r" => Color::Red,
        "y" => Color::Yellow,
        "g" => Color::Green,
        "b" => Color::Blue,
        "v" => Color::Violet,
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

/// The first round, in play, with these hands and `leader` to lead (the dealer at its right).
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
        won: vec![Vec::new(); n],
        void: vec![[false; 5]; n],
        scores: vec![0; n],
        last_round: None,
    }
}
