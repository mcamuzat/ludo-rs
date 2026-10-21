//! The board [B]: an 11 × 11 square with its corners cut, 109 cells.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// A cell, numbered in reading order (top to bottom, left to right).
pub type CellId = u8;

pub const SIZE: u8 = 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Lemon,
    Cherry,
    BlackPot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Cell {
    pub row: u8,
    pub col: u8,
    pub value: u8,
    pub kind: Kind,
}

/// The four directions of a line, as [R] lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dir {
    Row,
    Column,
    Diagonal,
    AntiDiagonal,
}

pub const DIRS: [Dir; 4] = [Dir::Row, Dir::Column, Dir::Diagonal, Dir::AntiDiagonal];

impl Dir {
    /// (row, column) step to the next cell of the line.
    pub fn step(self) -> (i8, i8) {
        match self {
            Dir::Row => (0, 1),
            Dir::Column => (1, 0),
            Dir::Diagonal => (1, 1),
            Dir::AntiDiagonal => (1, -1),
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// Five consecutive cells of a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub dir: Dir,
    pub cells: [CellId; 5],
}

/// [B] Values read off page 4, top to bottom, left to right; each row is
/// centred in the 11 columns.
const ROWS: [&[u8]; 11] = [
    &[19, 8, 3, 7, 5, 7, 17],
    &[1, 12, 17, 9, 2, 8, 13, 11, 1],
    &[19, 11, 13, 10, 3, 20, 4, 9, 14, 12, 17],
    &[2, 14, 16, 2, 4, 23, 3, 1, 15, 21, 2],
    &[5, 15, 4, 6, 21, 18, 21, 5, 3, 16, 6],
    &[1, 2, 10, 22, 24, 23, 24, 22, 10, 2, 1],
    &[6, 16, 3, 5, 21, 18, 21, 6, 4, 15, 5],
    &[2, 21, 15, 1, 3, 23, 4, 2, 16, 14, 2],
    &[17, 12, 14, 9, 4, 20, 3, 10, 13, 11, 19],
    &[1, 11, 13, 8, 2, 9, 17, 12, 1],
    &[17, 7, 5, 7, 3, 8, 19],
];

/// [R] The kind of a cell bearing `value`: lemons 1 to 6; cherries 7 to 12,
/// 15, 16, 18, 20 and 24; Black-Pots 13, 14, 17, 19, 21, 22 and 23.
pub fn kind_of(value: u8) -> Option<Kind> {
    match value {
        1..=6 => Some(Kind::Lemon),
        7..=12 | 15 | 16 | 18 | 20 | 24 => Some(Kind::Cherry),
        13 | 14 | 17 | 19 | 21 | 22 | 23 => Some(Kind::BlackPot),
        _ => None,
    }
}

struct Layout {
    cells: Vec<Cell>,
    /// `grid[row][col]`.
    grid: [[Option<CellId>; SIZE as usize]; SIZE as usize],
    windows: Vec<Window>,
}

fn layout() -> &'static Layout {
    static LAYOUT: OnceLock<Layout> = OnceLock::new();
    LAYOUT.get_or_init(|| {
        let mut cells = Vec::new();
        let mut grid = [[None; SIZE as usize]; SIZE as usize];
        for (r, row) in ROWS.iter().enumerate() {
            let offset = (usize::from(SIZE) - row.len()) / 2;
            for (i, &value) in row.iter().enumerate() {
                grid[r][offset + i] = Some(cells.len() as CellId);
                cells.push(Cell {
                    row: r as u8,
                    col: (offset + i) as u8,
                    value,
                    kind: kind_of(value).expect("every board value has a kind"),
                });
            }
        }
        let at = |r: i8, c: i8| -> Option<CellId> {
            let (r, c) = (usize::try_from(r).ok()?, usize::try_from(c).ok()?);
            *grid.get(r)?.get(c)?
        };
        let mut windows = Vec::new();
        for dir in DIRS {
            let (dr, dc) = dir.step();
            for cell in &cells {
                let (r, c) = (cell.row as i8, cell.col as i8);
                let run: Option<Vec<CellId>> = (0..5).map(|k| at(r + dr * k, c + dc * k)).collect();
                if let Some(run) = run {
                    windows.push(Window {
                        dir,
                        cells: run.try_into().expect("five cells"),
                    });
                }
            }
        }
        Layout {
            cells,
            grid,
            windows,
        }
    })
}

pub fn cells() -> &'static [Cell] {
    &layout().cells
}

/// The cell at (`row`, `col`), if the board has one there.
pub fn cell_at(row: i8, col: i8) -> Option<CellId> {
    let (r, c) = (usize::try_from(row).ok()?, usize::try_from(col).ok()?);
    *layout().grid.get(r)?.get(c)?
}

/// Every run of five consecutive cells, by direction then by first cell.
pub fn windows() -> &'static [Window] {
    &layout().windows
}

/// The board for the site to draw.
pub fn board_json() -> serde_json::Value {
    serde_json::json!({ "cells": cells() })
}
