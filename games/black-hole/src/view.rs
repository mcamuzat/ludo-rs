use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::{Action, Finish, Order, State, Tile};

/// Everything is in view but the order of the pools in the Random order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    pub order: Order,
    pub cells: Vec<Option<Tile>>,
    /// Each seat's tiles still to lay, smallest first.
    pub remaining: Vec<Vec<u8>>,
    /// The tile the seat to move must lay (none in the free order).
    pub next: Option<u8>,
    pub turn: Option<Seat>,
    pub last: Option<u8>,
    pub finish: Option<Finish>,
    /// The cells around the hole, once the game is over.
    pub around: Vec<u8>,
    pub legal: Vec<Action>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < 2 => Some(seat),
            _ => None,
        };
        let remaining = s
            .hands
            .iter()
            .map(|h| {
                let mut h = h.clone();
                h.sort_unstable();
                h
            })
            .collect();
        Self {
            me,
            order: s.config.order,
            cells: s.cells.clone(),
            remaining,
            next: s.turn.and_then(|t| s.next_tile(t)),
            turn: s.turn,
            last: s.last,
            finish: s.finish.clone(),
            around: s
                .finish
                .as_ref()
                .map(|f| crate::neighbours(f.hole))
                .unwrap_or_default(),
            legal: me.map(|m| s.moves(m)).unwrap_or_default(),
        }
    }
}
