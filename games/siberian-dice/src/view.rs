use ludo_core::Viewer;
use serde::Serialize;

use crate::board::cells;
use crate::{is_perfect, perfect_count, End, LastMove, State, Step, QUIET_ROUNDS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DieView {
    pub owner: u8,
    pub value: u8,
    pub perfect: bool,
}

/// A cell with its printed coordinates [R] (`ab` on page 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CellView {
    pub a: u8,
    pub b: u8,
    pub die: Option<DieView>,
}

/// Everything but the rolls to come is public.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub cells: Vec<CellView>,
    pub reserve: [u8; 2],
    pub perfect: [u8; 2],
    pub turn: u8,
    pub step: Step,
    pub last: Option<LastMove>,
    /// Rounds of the table without a new record left before the draw.
    pub quiet_left: u32,
    pub end: Option<End>,
    /// The seats ranked first once the game is over (both on a draw).
    pub winners: Vec<u8>,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        View {
            cells: cells()
                .iter()
                .enumerate()
                .map(|(id, c)| CellView {
                    a: c.a,
                    b: c.b,
                    die: s.board[id].map(|d| DieView {
                        owner: d.owner,
                        value: d.value,
                        perfect: is_perfect(&s.board, id as u8),
                    }),
                })
                .collect(),
            reserve: s.reserve,
            perfect: [perfect_count(&s.board, 0), perfect_count(&s.board, 1)],
            turn: s.turn.0,
            step: s.step,
            last: s.last,
            quiet_left: (2 * QUIET_ROUNDS).saturating_sub(s.quiet).div_ceil(2),
            end: s.end,
            winners: match s.end {
                None => Vec::new(),
                Some(End::Won { seat }) => vec![seat],
                Some(End::BothSix | End::QuietLimit) => vec![0, 1],
            },
        }
    }
}
