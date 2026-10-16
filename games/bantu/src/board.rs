//! The board as a graph, extracted from the vector pages of the rules
//! (`scripts/bantu-board.py`) into `board_data.rs`.

use std::collections::VecDeque;
use std::sync::OnceLock;

use serde::Serialize;

mod data {
    include!("board_data.rs");
}

pub use data::{CELLS, LINKS, ROWS, START_LINES};

/// Index into `CELLS`.
pub type Cell = u8;

/// A circle of the board: where it is drawn (0..1000 square, line 1 at the
/// bottom, line 4 at the left, the finish bottom left) and whether it bears
/// one of the four arrows into the finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CellDef {
    pub x: i16,
    pub y: i16,
    pub arrow: bool,
}

/// What a cell belongs to: a start line (piece number), a brown row, or
/// nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Group {
    Start(u8),
    Row(u16),
    Single,
}

struct Graph {
    group: Vec<Group>,
    /// Links forward, then neighbours along a row or a start line.
    next: Vec<Vec<Cell>>,
    distance: Vec<u8>,
}

fn graph() -> &'static Graph {
    static GRAPH: OnceLock<Graph> = OnceLock::new();
    GRAPH.get_or_init(|| {
        let n = CELLS.len();
        let mut group = vec![Group::Single; n];
        let mut next: Vec<Vec<Cell>> = vec![Vec::new(); n];
        for &(a, b) in LINKS.iter() {
            next[usize::from(a)].push(b);
        }
        let lines = START_LINES
            .iter()
            .enumerate()
            .map(|(i, l)| (Group::Start(i as u8 + 1), &l[..]));
        let rows = ROWS
            .iter()
            .enumerate()
            .map(|(i, r)| (Group::Row(i as u16), *r));
        for (g, cells) in lines.chain(rows) {
            for &c in cells {
                group[usize::from(c)] = g;
            }
            for pair in cells.windows(2) {
                next[usize::from(pair[0])].push(pair[1]);
                next[usize::from(pair[1])].push(pair[0]);
            }
        }
        // Steps to the finish, against the moves: one from an arrow cell.
        let mut distance = vec![u8::MAX; n];
        let mut todo = VecDeque::new();
        for (i, c) in CELLS.iter().enumerate() {
            if c.arrow {
                distance[i] = 1;
                todo.push_back(i);
            }
        }
        while let Some(i) = todo.pop_front() {
            for (j, outs) in next.iter().enumerate() {
                if distance[j] == u8::MAX && outs.contains(&(i as Cell)) {
                    distance[j] = distance[i] + 1;
                    todo.push_back(j);
                }
            }
        }
        Graph {
            group,
            next,
            distance,
        }
    })
}

pub fn group(cell: Cell) -> Group {
    graph().group[usize::from(cell)]
}

/// Where a piece on `cell` may step: forward links, and both ways along its
/// row or start line.
pub fn neighbours(cell: Cell) -> impl Iterator<Item = Cell> {
    graph().next[usize::from(cell)].iter().copied()
}

/// Fewest steps from `cell` into the finish (1 from an arrow cell);
/// `u8::MAX` when the finish cannot be reached.
pub fn distance_to_finish(cell: Cell) -> u8 {
    graph().distance[usize::from(cell)]
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn all_cells() -> impl Iterator<Item = Cell> {
        (0..CELLS.len()).map(|i| i as Cell)
    }

    #[test]
    fn four_start_lines_of_four() {
        let cells: HashSet<Cell> = START_LINES.iter().flatten().copied().collect();
        assert_eq!(cells.len(), 16);
        assert!(cells.iter().all(|&c| usize::from(c) < CELLS.len()));
        assert!(ROWS.iter().all(|r| r.iter().all(|c| !cells.contains(c))));
    }

    #[test]
    fn four_arrow_cells() {
        assert_eq!(CELLS.iter().filter(|c| c.arrow).count(), 4);
    }

    #[test]
    fn every_start_cell_reaches_the_finish() {
        for &c in START_LINES.iter().flatten() {
            assert!(distance_to_finish(c) < u8::MAX, "cell {c}");
        }
    }

    #[test]
    fn every_cell_is_reachable() {
        let mut seen: HashSet<Cell> = START_LINES.iter().flatten().copied().collect();
        let mut todo: Vec<Cell> = seen.iter().copied().collect();
        while let Some(c) = todo.pop() {
            for n in neighbours(c) {
                if seen.insert(n) {
                    todo.push(n);
                }
            }
        }
        assert!(!CELLS.is_empty());
        assert_eq!(seen.len(), CELLS.len());
    }

    #[test]
    fn progress_is_acyclic() {
        // Kahn's algorithm on the links alone.
        let mut incoming = vec![0usize; CELLS.len()];
        for &(_, b) in LINKS.iter() {
            incoming[usize::from(b)] += 1;
        }
        let mut todo: Vec<Cell> = all_cells()
            .filter(|&c| incoming[usize::from(c)] == 0)
            .collect();
        let mut done = 0;
        while let Some(c) = todo.pop() {
            done += 1;
            for &(_, b) in LINKS.iter().filter(|(a, _)| *a == c) {
                incoming[usize::from(b)] -= 1;
                if incoming[usize::from(b)] == 0 {
                    todo.push(b);
                }
            }
        }
        assert_eq!(done, CELLS.len());
    }

    /// Cells reachable from `from` without stepping on `avoid`.
    fn reach(from: &[Cell], avoid: &HashSet<Cell>) -> HashSet<Cell> {
        let mut seen: HashSet<Cell> = from.iter().copied().collect();
        let mut todo = from.to_vec();
        while let Some(c) = todo.pop() {
            for n in neighbours(c) {
                if !avoid.contains(&n) && seen.insert(n) {
                    todo.push(n);
                }
            }
        }
        seen
    }

    #[test]
    fn line_n_flows_to_the_next() {
        let line = |n: usize| -> HashSet<Cell> { START_LINES[n - 1].iter().copied().collect() };
        let arrows: HashSet<Cell> = all_cells()
            .filter(|&c| CELLS[usize::from(c)].arrow)
            .collect();
        // From line 1, the finish without any other line.
        let others: HashSet<Cell> = (2..=4).flat_map(line).collect();
        assert!(reach(&START_LINES[0], &others)
            .iter()
            .any(|c| arrows.contains(c)));
        // From line n (2 to 4): line n - 1 is reached, but neither the finish
        // nor any other line before it.
        for n in 2..=4 {
            let mut blocked: HashSet<Cell> = (1..=4).filter(|&k| k != n).flat_map(line).collect();
            blocked.extend(&arrows);
            let reached = reach(&START_LINES[n - 1], &blocked);
            let next = line(n - 1);
            assert!(
                reached
                    .iter()
                    .any(|c| neighbours(*c).any(|x| next.contains(&x))),
                "line {n} does not lead to line {}",
                n - 1
            );
            assert!(reached.iter().all(|c| !arrows.contains(c)));
        }
    }

    #[test]
    fn groups_cover_lines_and_rows() {
        assert_eq!(group(START_LINES[2][0]), Group::Start(3));
        assert!(matches!(group(ROWS[0][0]), Group::Row(0)));
    }
}
