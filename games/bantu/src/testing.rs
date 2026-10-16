//! Hand-built positions for tests.

use ludo_core::Seat;

use crate::{Config, Spot, State};

/// A normal game of `players` where only the listed pieces (seat, number,
/// spot) are out of the finish; `turn` to move.
pub fn position(players: u8, pieces: &[(u8, u8, Spot)], turn: u8) -> State {
    let config = Config::default();
    let mut all = vec![vec![Spot::Finished; usize::from(config.pieces())]; usize::from(players)];
    for &(seat, piece, spot) in pieces {
        all[usize::from(seat)][usize::from(piece) - 1] = spot;
    }
    State {
        config,
        players,
        pieces: all,
        turn: Some(Seat(turn)),
        quiet: 0,
        last: None,
        end: None,
    }
}
