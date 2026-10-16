//! How far a piece moves, where it may land, and who is safe.

use ludo_core::Seat;

use crate::board::{distance_to_finish, group, neighbours, Cell, Group, CELLS, START_LINES};
use crate::{Dest, Spot, State};

impl State {
    /// The piece on `cell`, as (seat, piece number).
    pub fn occupant(&self, cell: Cell) -> Option<(Seat, u8)> {
        self.pieces.iter().enumerate().find_map(|(seat, pieces)| {
            pieces
                .iter()
                .position(|&p| p == Spot::On { cell })
                .map(|i| (Seat(seat as u8), i as u8 + 1))
        })
    }

    /// Numbers of the pieces standing in `g` (a row or a start line).
    fn numbers_in(&self, g: Group) -> Vec<u8> {
        self.pieces
            .iter()
            .flat_map(|pieces| pieces.iter().enumerate())
            .filter_map(|(i, p)| match p {
                Spot::On { cell } if group(*cell) == g => Some(i as u8 + 1),
                _ => None,
            })
            .collect()
    }
}

/// [R] A piece alone in its row moves as far as its number; two or more
/// pieces in the same row or start line, whatever their colour, each move
/// the sum of all their numbers.
pub fn move_length(s: &State, seat: Seat, piece: u8) -> u8 {
    let Some(Spot::On { cell }) = s.pieces[seat.index()].get(usize::from(piece) - 1).copied()
    else {
        return piece;
    };
    match group(cell) {
        Group::Single => piece,
        g => {
            let numbers = s.numbers_in(g);
            if numbers.len() >= 2 {
                numbers.iter().sum()
            } else {
                piece
            }
        }
    }
}

/// [R] A piece cannot be taken on a start line, nor in a row it shares with
/// another piece of any colour.
pub fn is_protected(s: &State, cell: Cell) -> bool {
    match group(cell) {
        Group::Start(_) => true,
        g @ Group::Row(_) => s.numbers_in(g).len() >= 2,
        Group::Single => false,
    }
}

/// Where `piece` of `seat` may go this turn.
pub fn destinations(s: &State, seat: Seat, piece: u8) -> Vec<Dest> {
    let mut out = Vec::new();
    match s.pieces[seat.index()][usize::from(piece) - 1] {
        Spot::Finished => {}
        // [R] Out onto any free cell of the start line of its number.
        Spot::Home => out.extend(
            START_LINES[usize::from(piece) - 1]
                .iter()
                .filter(|&&c| s.occupant(c).is_none())
                .map(|&c| Dest::Cell(c)),
        ),
        Spot::On { cell } => {
            let steps = move_length(s, seat, piece);
            // [R] The finish is reached from an arrow cell, without an exact
            // count: a piece already there goes in with any step left.
            if CELLS[usize::from(cell)].arrow {
                out.push(Dest::Finish);
            }
            let mut path = vec![cell];
            walk(s, seat, cell, steps, &mut path, &mut out);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// [R] Exactly `left` steps forward or sideways along a row, never through
/// a cell already crossed this turn nor over any piece.
fn walk(s: &State, seat: Seat, at: Cell, left: u8, path: &mut Vec<Cell>, out: &mut Vec<Dest>) {
    for next in neighbours(at) {
        if path.contains(&next) {
            continue;
        }
        let occupant = s.occupant(next);
        if left == 1 {
            // [R] Landing on an opponent takes it, unless it is safe.
            let free = match occupant {
                None => true,
                Some((owner, _)) => owner != seat && !is_protected(s, next),
            };
            if free {
                out.push(Dest::Cell(next));
            }
            continue;
        }
        if occupant.is_some() {
            continue;
        }
        if CELLS[usize::from(next)].arrow {
            out.push(Dest::Finish);
        }
        path.push(next);
        walk(s, seat, next, left - 1, path, out);
        path.pop();
    }
}

/// For the ranking: pieces in the finish, and the steps the others still
/// need (a piece at home: one more than the nearest cell of its line).
pub fn progress(s: &State, seat: Seat) -> (u8, u32) {
    let mut finished = 0;
    let mut distance = 0;
    for (i, spot) in s.pieces[seat.index()].iter().enumerate() {
        match spot {
            Spot::Finished => finished += 1,
            Spot::On { cell } => distance += u32::from(distance_to_finish(*cell)),
            Spot::Home => {
                let nearest = START_LINES[i].iter().map(|&c| distance_to_finish(c)).min();
                distance += 1 + u32::from(nearest.expect("four cells per line"));
            }
        }
    }
    (finished, distance)
}
