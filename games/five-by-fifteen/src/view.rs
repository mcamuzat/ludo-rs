use serde::Serialize;

use crate::{Color, Route, State, Status, Tile, COLS, ROWS};

/// Everything is face up: one view for all.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub route: Route,
    pub min_run: u8,
    pub anchors: [Color; ROWS],
    pub cells: [[Option<Tile>; COLS]; ROWS],
    pub status: Status,
    pub moves: u32,
    pub repeated: bool,
    /// Tiles in place from the left of each row (0 to 14).
    pub sorted: [u8; ROWS],
    /// Every 15 led by its 13 and 14: the locomotive.
    pub locomotives: Vec<Locomotive>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Locomotive {
    pub row: u8,
    /// The column of the 15.
    pub col: u8,
    /// Action b may move this train now.
    pub ready: bool,
}

impl View {
    pub(crate) fn of(s: &State) -> View {
        let mut locomotives = Vec::new();
        for r in 0..ROWS {
            for c in 0..COLS {
                if s.cells[r][c].is_some_and(|t| t.number == 15) && s.run_ending_at(r, c) >= 3 {
                    locomotives.push(Locomotive {
                        row: r as u8,
                        col: c as u8,
                        ready: s.shift_run(r, c + 1).is_some(),
                    });
                }
            }
        }
        View {
            route: s.config.route,
            min_run: s.config.route.min_run() as u8,
            anchors: s.anchors,
            cells: s.cells,
            status: s.status,
            moves: s.moves,
            repeated: s.repeated,
            sorted: std::array::from_fn(|r| s.sorted(r) as u8),
            locomotives,
        }
    }
}
