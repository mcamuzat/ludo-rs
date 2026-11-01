//! The triangular board: 6 rows of 1 to 6 cells, numbered row by row from
//! the apex, packed like balls.

use serde::{Deserialize, Serialize};

pub const ROWS: u8 = 6;
/// [B] 21 cells.
pub const CELLS: usize = 21;

/// A cell by its row (0 at the apex) and its place in the row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pos {
    pub row: u8,
    pub col: u8,
}

pub fn pos(cell: u8) -> Pos {
    let mut row = 0;
    while (row + 1) * (row + 2) / 2 <= cell {
        row += 1;
    }
    Pos {
        row,
        col: cell - row * (row + 1) / 2,
    }
}

fn index(row: i8, col: i8) -> Option<u8> {
    (row >= 0 && row < ROWS as i8 && col >= 0 && col <= row)
        .then(|| (row * (row + 1) / 2 + col) as u8)
}

/// The cells touching `cell`: two in its row, two above, two below.
pub fn neighbours(cell: u8) -> Vec<u8> {
    let Pos { row, col } = pos(cell);
    let (r, c) = (row as i8, col as i8);
    [
        (r, c - 1),
        (r, c + 1),
        (r - 1, c - 1),
        (r - 1, c),
        (r + 1, c),
        (r + 1, c + 1),
    ]
    .into_iter()
    .filter_map(|(r, c)| index(r, c))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_have_two_neighbours_and_the_middle_six() {
        assert_eq!(neighbours(0), vec![1, 2]);
        assert_eq!(neighbours(15).len(), 2);
        assert_eq!(neighbours(20).len(), 2);
        // Row 3, second cell: fully surrounded.
        assert_eq!(pos(7), Pos { row: 3, col: 1 });
        assert_eq!(neighbours(7).len(), 6);
        assert_eq!(pos(20), Pos { row: 5, col: 5 });
    }

    #[test]
    fn neighbourhood_is_symmetric() {
        for a in 0..CELLS as u8 {
            for b in neighbours(a) {
                assert!(neighbours(b).contains(&a), "{a} {b}");
            }
        }
    }
}
