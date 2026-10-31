//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Card, Config, Phase, State, Suit, Who};

/// A card from its short name: `r7`, `g12`, `b1`, `y13`.
pub fn c(name: &str) -> Card {
    let (suit, value) = name.split_at(1);
    let suit = match suit {
        "r" => Suit::Red,
        "g" => Suit::Green,
        "b" => Suit::Blue,
        "y" => Suit::Yellow,
        other => panic!("unknown suit {other}"),
    };
    Card {
        suit,
        value: value.parse().expect("a value"),
    }
}

pub fn cards(names: &str) -> Vec<Card> {
    names.split_whitespace().map(c).collect()
}

/// A round in play (baskets started, all empty), with these hands, `leader`
/// to lead and seat 0 dealing. At two players, `dummy` is the dummy's pile,
/// top first.
pub fn table(hands: &[&str], dummy: &str, leader: u8) -> State {
    let n = hands.len();
    State {
        config: Config::default(),
        players: n as u8,
        seed: 0,
        round: 0,
        dealer: Seat(0),
        phase: Phase::Trick,
        hands: hands.iter().map(|h| cards(h)).collect(),
        dummy: cards(dummy),
        set_aside: Vec::new(),
        chosen: vec![None; n],
        revealed: vec![None; n],
        baskets: vec![0; n],
        center: 0,
        triples: 0,
        kings: vec![true; n],
        first_leader: Seat(leader),
        leader: Who::Seat(Seat(leader)),
        trick: Vec::new(),
        turn: Some(Who::Seat(Seat(leader))),
        last_trick: None,
        void: vec![[false; 4]; n],
        gained: vec![0; n],
        scores: vec![0; n],
        last_round: None,
    }
}
