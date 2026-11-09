use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::{home_row, Action, End, Last, State, Wizard, QUIET_LIMIT};

/// Everything is in view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    /// 25 cells, row by row from yellow's end (top).
    pub cells: Vec<Option<Wizard>>,
    pub reserve: [u8; 2],
    /// The wizards each seat has captured.
    pub captured: [u8; 2],
    /// Each seat's end of the board (row number).
    pub home_rows: [u8; 2],
    pub turn: Option<Seat>,
    /// Turns left before the draw for lack of captures.
    pub quiet_left: u16,
    pub last: Option<Last>,
    pub end: Option<End>,
    pub legal: Vec<Action>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < 2 => Some(seat),
            _ => None,
        };
        Self {
            me,
            cells: s.cells.clone(),
            reserve: s.reserve,
            captured: s.captured,
            home_rows: [home_row(Seat(0)), home_row(Seat(1))],
            turn: s.turn,
            quiet_left: QUIET_LIMIT.saturating_sub(s.quiet),
            last: s.last.clone(),
            end: s.end,
            legal: me.map(|m| s.moves(m)).unwrap_or_default(),
        }
    }
}
