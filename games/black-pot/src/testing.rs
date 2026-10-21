//! Hand-built positions for tests.

use ludo_core::{Game, Seat};

use crate::board::{cells, CellId};
use crate::{BlackPot, Config, State};

/// A game of `players` with the pawns listed (cell, team) on the board,
/// not yet counted in any line, and their team's stock reduced; `dice`
/// rolled by the last seat of `queue`, which plays it in this order. Turns
/// of players without a move then pass by themselves.
pub fn position(players: u8, owners: &[(CellId, u8)], dice: [u8; 3], queue: &[u8]) -> State {
    let mut s = BlackPot::new(&Config::default(), players, 0);
    s.owner = vec![None; cells().len()];
    s.counted = vec![[false; 4]; cells().len()];
    s.pawns_left = [crate::PAWNS; 2];
    for &(cell, team) in owners {
        s.owner[usize::from(cell)] = Some(team);
        s.pawns_left[usize::from(team)] = s.pawns_left[usize::from(team)].saturating_sub(1);
    }
    s.dice = dice;
    s.queue = queue.iter().map(|&seat| Seat(seat)).collect();
    s.roller = *s.queue.last().expect("someone plays");
    s.last = vec![None; usize::from(players)];
    s.quiet = 0;
    s.turns = 0;
    s.end = None;
    s.settle();
    s
}

/// The first free cell bearing `value`.
pub fn free_cell(s: &State, value: u8) -> CellId {
    cells()
        .iter()
        .enumerate()
        .position(|(id, c)| c.value == value && s.owner[id].is_none())
        .expect("a free cell of that value") as CellId
}
