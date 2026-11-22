//! Positions built by hand, for tests.

use crate::{Color, Config, Route, State, Tile, COLS, ROWS};

/// A game at this position, never played: its status follows from it.
pub fn state(anchors: [Color; ROWS], cells: [[Option<Tile>; COLS]; ROWS], route: Route) -> State {
    State::fresh(Config { route }, anchors, cells)
}
