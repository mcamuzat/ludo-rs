//! [R] page 3: 37 hexagonal cells numbered `ab`, with `a, b` from 1 to 7
//! and `|a - b| <= 3`.

use std::sync::OnceLock;

/// Index of a cell in `cells()`.
pub type CellId = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub a: u8,
    pub b: u8,
}

struct Board {
    cells: Vec<Cell>,
    neighbors: Vec<Vec<CellId>>,
}

fn board() -> &'static Board {
    static BOARD: OnceLock<Board> = OnceLock::new();
    BOARD.get_or_init(|| {
        let cells: Vec<Cell> = (1..=7u8)
            .flat_map(|a| (1..=7u8).map(move |b| Cell { a, b }))
            .filter(|c| c.a.abs_diff(c.b) <= 3)
            .collect();
        let find = |a: i8, b: i8| {
            cells
                .iter()
                .position(|c| i8::try_from(c.a) == Ok(a) && i8::try_from(c.b) == Ok(b))
                .map(|i| i as CellId)
        };
        // [R] Six sides: along `a`, along `b`, and along both at once.
        const STEPS: [(i8, i8); 6] = [(-1, -1), (1, 1), (0, -1), (1, 0), (0, 1), (-1, 0)];
        let neighbors = cells
            .iter()
            .map(|c| {
                STEPS
                    .iter()
                    .filter_map(|&(da, db)| find(c.a as i8 + da, c.b as i8 + db))
                    .collect()
            })
            .collect();
        Board { cells, neighbors }
    })
}

/// The 37 cells, sorted by `(a, b)`.
pub fn cells() -> &'static [Cell] {
    &board().cells
}

/// The cell numbered `ab` on page 3 of [R]. Panics if there is none.
pub fn at(ab: u8) -> CellId {
    cells()
        .iter()
        .position(|c| c.a == ab / 10 && c.b == ab % 10)
        .unwrap_or_else(|| panic!("no cell {ab}")) as CellId
}

/// The number `ab` printed on the cell.
pub fn label(id: CellId) -> u8 {
    let c = cells()[usize::from(id)];
    10 * c.a + c.b
}

/// The cells sharing a side with `id`.
pub fn neighbors(id: CellId) -> &'static [CellId] {
    &board().neighbors[usize::from(id)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thirty_seven_cells() {
        assert_eq!(cells().len(), 37);
    }

    #[test]
    fn rows_of_the_printed_board() {
        // [R] page 3: top, middle and bottom rows.
        for ab in [41, 52, 63, 74, 11, 22, 33, 44, 55, 66, 77, 14, 25, 36, 47] {
            assert_eq!(label(at(ab)), ab);
        }
    }

    fn sorted_neighbors(ab: u8) -> Vec<u8> {
        let mut v: Vec<u8> = neighbors(at(ab)).iter().map(|&c| label(c)).collect();
        v.sort_unstable();
        v
    }

    #[test]
    fn neighbors_of_center_edge_and_corner() {
        assert_eq!(sorted_neighbors(44), vec![33, 34, 43, 45, 54, 55]);
        assert_eq!(sorted_neighbors(11), vec![12, 21, 22]);
        assert_eq!(sorted_neighbors(41), vec![31, 42, 52]);
        assert_eq!(sorted_neighbors(22), vec![11, 12, 21, 23, 32, 33]);
    }

    #[test]
    fn adjacency_is_symmetric() {
        for c in 0..cells().len() as CellId {
            for &d in neighbors(c) {
                assert!(neighbors(d).contains(&c), "{} {}", label(c), label(d));
            }
        }
    }
}
