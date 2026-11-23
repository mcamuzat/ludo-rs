use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::{values, Action, Dir, Finish, State};

/// A laid domino as drawn: `first` is the value on the anchor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlacedView {
    pub row: i8,
    pub col: i8,
    pub dir: Dir,
    pub first: u8,
    pub second: u8,
    /// `None` for the starting square.
    pub by: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HandDomino {
    pub id: u8,
    pub a: u8,
    pub b: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LastMove {
    pub seat: Seat,
    /// Index in `placed`; `None` for a pass.
    pub placed: Option<usize>,
    pub points: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LegalMove {
    pub action: Action,
    pub points: u32,
}

/// The table, one's own hand, and the sizes of the others.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    pub placed: Vec<PlacedView>,
    pub hand: Vec<HandDomino>,
    pub hand_sizes: Vec<u8>,
    pub scores: Vec<i32>,
    pub turn: Option<Seat>,
    pub first: Seat,
    pub last_round: bool,
    pub last: Option<LastMove>,
    pub finish: Option<Finish>,
    /// The placements of the seat to play, with their points, for that seat
    /// only.
    pub legal: Vec<LegalMove>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < s.hands.len() => Some(seat),
            _ => None,
        };
        let placed = s
            .board
            .placed()
            .iter()
            .map(|p| {
                let (first, second) = p.at.sides();
                PlacedView {
                    row: p.at.row,
                    col: p.at.col,
                    dir: p.at.dir,
                    first,
                    second,
                    by: p.by,
                }
            })
            .collect::<Vec<_>>();
        let last = s.last.map(|(seat, action, points)| LastMove {
            seat,
            placed: matches!(action, Action::Place(_)).then(|| placed.len() - 1),
            points,
        });
        Self {
            me,
            hand: me
                .map(|m| {
                    s.hands[m.index()]
                        .iter()
                        .map(|&id| {
                            let (a, b) = values(id);
                            HandDomino { id, a, b }
                        })
                        .collect()
                })
                .unwrap_or_default(),
            hand_sizes: s.hands.iter().map(|h| h.len() as u8).collect(),
            scores: s.scores.clone(),
            turn: s.turn,
            first: s.first,
            last_round: s.last_round,
            last,
            finish: s.finish.clone(),
            legal: me
                .map(|m| {
                    s.moves(m)
                        .into_iter()
                        .map(|(action, points)| LegalMove { action, points })
                        .collect()
                })
                .unwrap_or_default(),
            placed,
        }
    }
}
