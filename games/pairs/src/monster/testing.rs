//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{ante, Phase, State};
use crate::deck::Supply;
use crate::testing::ranks;

/// Game 1 of `games`, round 1, everyone anted and dealt `hands`; seat 0
/// holds the button, seat 1 acts first.
pub fn table(hands: &[&[u8]], games: u8) -> State {
    let n = hands.len();
    let a = ante(n as u8);
    State {
        players: n as u8,
        games_total: games,
        game_no: 1,
        round_no: 1,
        deals: 1,
        seed: 0,
        supply: Supply::from_parts(Vec::new(), Vec::new(), Vec::new(), 0, 0),
        hands: hands.iter().map(|h| ranks(h)).collect(),
        discards: vec![Vec::new(); n],
        folded: vec![Vec::new(); n],
        in_game: vec![true; n],
        in_round: vec![true; n],
        acted: vec![false; n],
        button: Seat(0),
        turn: Some(Seat(1)),
        phase: Phase::Act,
        pot: a * n as u32,
        chips: vec![-(a as i32); n],
        events: Vec::new(),
    }
}
