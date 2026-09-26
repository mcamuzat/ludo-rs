//! Hexagonal board of side 5 (61 cells) in axial coordinates.

use serde::{Deserialize, Serialize};

pub const RADIUS: i8 = 4;
pub const CELL_COUNT: usize = 61;
/// Index of the first cell of each row r = -4..=4 (row lengths 5,6,7,8,9,8,7,6,5).
const ROW_START: [usize; 9] = [0, 5, 11, 18, 26, 35, 43, 50, 56];
/// The three line directions; each line is walked both ways.
pub const DIRECTIONS: [(i8, i8); 3] = [(1, 0), (0, 1), (1, -1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell {
    pub q: i8,
    pub r: i8,
}

impl Cell {
    pub const fn new(q: i8, r: i8) -> Self {
        Self { q, r }
    }

    pub fn is_on_board(self) -> bool {
        self.q.abs() <= RADIUS && self.r.abs() <= RADIUS && (self.q + self.r).abs() <= RADIUS
    }

    /// Position in `all_cells()` order, or `None` off the board.
    pub fn index(self) -> Option<usize> {
        if !self.is_on_board() {
            return None;
        }
        let q_min = (-RADIUS).max(-RADIUS - self.r);
        Some(ROW_START[(self.r + RADIUS) as usize] + (self.q - q_min) as usize)
    }

    fn step(self, (dq, dr): (i8, i8), k: i8) -> Cell {
        Cell::new(self.q + dq * k, self.r + dr * k)
    }
}

/// Every cell, row by row (r ascending, then q ascending).
pub fn all_cells() -> impl Iterator<Item = Cell> {
    (-RADIUS..=RADIUS).flat_map(|r| {
        let q_min = (-RADIUS).max(-RADIUS - r);
        let q_max = RADIUS.min(RADIUS - r);
        (q_min..=q_max).map(move |q| Cell::new(q, r))
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
    Red,
}

impl Color {
    /// Turn order [R]: White starts, then Black, then Red.
    pub const TURN_ORDER: [Color; 3] = [Color::White, Color::Black, Color::Red];
}

/// What placing a stone does [R]: 4 or more in a row wins; exactly 3
/// without a 4 loses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Wins,
    Loses,
    Neutral,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    stones: Vec<Option<Color>>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            stones: vec![None; CELL_COUNT],
        }
    }
}

impl Board {
    /// `None` for an empty cell or a cell off the board.
    pub fn get(&self, cell: Cell) -> Option<Color> {
        cell.index().and_then(|i| self.stones[i])
    }

    pub fn set(&mut self, cell: Cell, stone: Option<Color>) {
        let i = cell.index().expect("cell on the board");
        self.stones[i] = stone;
    }

    pub fn is_empty(&self, cell: Cell) -> bool {
        cell.is_on_board() && self.get(cell).is_none()
    }

    pub fn is_full(&self) -> bool {
        self.stones.iter().all(Option::is_some)
    }

    pub fn empty_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        all_cells().filter(|&c| self.get(c).is_none())
    }

    /// Length of the run of `color` through `cell` along `dir`, counting `cell` as `color`.
    fn run_len(&self, cell: Cell, color: Color, dir: (i8, i8)) -> usize {
        let mut n = 1;
        for sign in [-1i8, 1] {
            let mut k = 1;
            while self.get(cell.step(dir, sign * k)) == Some(color) {
                n += 1;
                k += 1;
            }
        }
        n
    }

    fn run(&self, cell: Cell, color: Color, dir: (i8, i8)) -> Vec<Cell> {
        let mut back = 1;
        while self.get(cell.step(dir, -back)) == Some(color) {
            back += 1;
        }
        let mut forward = 1;
        while self.get(cell.step(dir, forward)) == Some(color) {
            forward += 1;
        }
        (1 - back..forward).map(|k| cell.step(dir, k)).collect()
    }

    /// What placing `color` on `cell` would do.
    pub fn effect(&self, cell: Cell, color: Color) -> Effect {
        let mut three = false;
        for dir in DIRECTIONS {
            match self.run_len(cell, color, dir) {
                n if n >= 4 => return Effect::Wins,
                3 => three = true,
                _ => {}
            }
        }
        if three {
            Effect::Loses
        } else {
            Effect::Neutral
        }
    }

    /// The runs through `cell` (counted as `color`) that decide the placement:
    /// the runs of 4 or more if any, otherwise the runs of exactly 3.
    pub fn deciding_lines(&self, cell: Cell, color: Color) -> Vec<Vec<Cell>> {
        let runs: Vec<Vec<Cell>> = DIRECTIONS
            .iter()
            .map(|&d| self.run(cell, color, d))
            .collect();
        if runs.iter().any(|r| r.len() >= 4) {
            runs.into_iter().filter(|r| r.len() >= 4).collect()
        } else {
            runs.into_iter().filter(|r| r.len() == 3).collect()
        }
    }
}
