//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Config, State, PLAYERS};
pub use crate::president::testing::{c, cards};

/// The first deal, in play: these hands, `leader` to lead (the dealer sits
/// at its right), nothing said yet.
pub fn table(hands: [&str; 4], leader: u8) -> State {
    State {
        config: Config::default(),
        seed: 0,
        deal_no: 0,
        dealer: Seat((leader + PLAYERS as u8 - 1) % PLAYERS as u8),
        hands: hands.iter().map(|h| cards(h)).collect(),
        trick: Vec::new(),
        last_trick: None,
        turn: Some(Seat(leader)),
        taken: [0; 4],
        voids: [[false; 4]; 4],
        said: false,
        talk: Vec::new(),
        scores: [0; 4],
        last_deal: None,
    }
}
