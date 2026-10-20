//! Hand-built tables for tests.

use ludo_core::Seat;

use crate::clues::{generate, Mode, Puzzle};
use crate::map::CELLS;
use crate::rules::{Phase, State, TableCache, Tokens};

/// A known normal-mode puzzle for `players` seats.
pub fn fixed_puzzle(players: u8) -> Puzzle {
    generate(Mode::Normal, players, 1)
}

/// Seat 0 to move after the setup, no token on the map.
pub fn table(players: u8, puzzle: Puzzle) -> State {
    State {
        players,
        mode: Mode::Normal,
        puzzle,
        tokens: vec![Tokens::default(); usize::from(CELLS)],
        phase: Phase::Turn,
        turn: Seat(0),
        first: Seat(0),
        setup_left: 0,
        log: Vec::new(),
        winner: None,
        cache: TableCache::default(),
    }
}
