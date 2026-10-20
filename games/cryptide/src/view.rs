//! What a seat sees: the map, every token, its own hint; the others' hints
//! and the habitat only once the game is over.

use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::clues::{hints, Hint, Mode};
use crate::map::{Cell, Square};
use crate::rules::{Event, Phase, State, Tokens};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Reveal {
    pub hints: Vec<Hint>,
    pub habitat: Cell,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct View {
    pub players: u8,
    pub mode: Mode,
    pub squares: Vec<Square>,
    pub tokens: Vec<Tokens>,
    pub phase: Phase,
    pub turn: Seat,
    /// The search pawn: the space of the latest question or search.
    pub focus: Option<Cell>,
    pub my_hint: Option<Hint>,
    /// For the player aid: every hint of the mode.
    pub hints: Vec<Hint>,
    pub log: Vec<crate::rules::Event>,
    pub reveal: Option<Reveal>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> View {
        let my_hint = match viewer {
            Viewer::Seat(seat) => s.puzzle.hints.get(seat.index()).copied(),
            Viewer::Spectator => None,
        };
        let focus = s.log.iter().rev().find_map(|e| match *e {
            Event::Asked { cell, .. } | Event::Searched { cell, .. } => Some(cell),
            _ => None,
        });
        let reveal = (s.phase == Phase::Over).then(|| Reveal {
            hints: s.puzzle.hints.clone(),
            habitat: s.puzzle.habitat,
        });
        View {
            players: s.players,
            mode: s.mode,
            squares: s.puzzle.board().squares,
            tokens: s.tokens.clone(),
            phase: s.phase.clone(),
            turn: s.turn,
            focus,
            my_hint,
            hints: hints(s.mode),
            log: s.log.clone(),
            reveal,
        }
    }
}
