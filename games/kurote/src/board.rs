//! The 5×5 board. Cells are numbered row by row from the top left (yellow's
//! side): cell = row × 5 + column.

pub const SIZE: u8 = 5;
pub const CELLS: usize = 25;

pub fn cell(row: u8, col: u8) -> u8 {
    row * SIZE + col
}

pub fn row(cell: u8) -> u8 {
    cell / SIZE
}

pub fn col(cell: u8) -> u8 {
    cell % SIZE
}

/// [K] The up to 8 cells surrounding `cell`, orthogonally or diagonally.
pub fn neighbours(cell: u8) -> impl Iterator<Item = u8> {
    let (r, c) = (i16::from(row(cell)), i16::from(col(cell)));
    (-1i16..=1)
        .flat_map(move |dr| (-1i16..=1).map(move |dc| (r + dr, c + dc)))
        .filter(move |&(nr, nc)| (nr, nc) != (r, c) && (0..5).contains(&nr) && (0..5).contains(&nc))
        .map(|(nr, nc)| (nr * 5 + nc) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_edges_and_centre() {
        assert_eq!(neighbours(cell(0, 0)).collect::<Vec<_>>(), vec![1, 5, 6]);
        assert_eq!(neighbours(cell(0, 2)).count(), 5);
        assert_eq!(neighbours(cell(2, 2)).count(), 8);
        assert_eq!(neighbours(cell(4, 4)).collect::<Vec<_>>(), vec![18, 19, 23]);
    }
}
