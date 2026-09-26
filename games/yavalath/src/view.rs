use ludo_core::Seat;
use serde::Serialize;

use crate::{all_cells, Cell, Color, State};

/// Everything is public in Yavalath: every viewer gets the same view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub cells: Vec<CellView>,
    pub seat_colors: Vec<Color>,
    /// Seat to move; `None` once the game is over.
    pub current: Option<Seat>,
    pub eliminated: Vec<Seat>,
    /// Cells the player to move must choose from to block the next player (3 players).
    pub forced: Vec<Cell>,
    pub last_move: Option<Cell>,
    /// Lines to highlight: 3s that eliminated someone, the 4 that won.
    pub lines: Vec<Vec<Cell>>,
    pub swapped: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CellView {
    pub q: i8,
    pub r: i8,
    pub stone: Option<Color>,
}

impl View {
    pub fn of(s: &State) -> Self {
        Self {
            cells: all_cells()
                .map(|c| CellView {
                    q: c.q,
                    r: c.r,
                    stone: s.board.get(c),
                })
                .collect(),
            seat_colors: s.seat_colors.clone(),
            current: s.current.map(|c| s.seat_of(c)),
            eliminated: s.eliminated.clone(),
            forced: s.forced_blocks(),
            last_move: s.last_move,
            lines: s.lines.clone(),
            swapped: s.swapped,
        }
    }
}
