use ludo_core::{Game, Seat, Viewer};
use serde::Serialize;

use crate::board::{Cell, CELLS};
use crate::{is_protected, move_length, Action, Bantu, End, LastMove, Spot, State, QUIET_ROUNDS};

/// Everything is public in Bantu: the whole position, plus what the board
/// needs to show without computing it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: u8,
    /// `pieces[seat][number - 1]`.
    pub pieces: Vec<Vec<Spot>>,
    pub turn: Option<Seat>,
    /// How far each piece of the player to move goes this turn (0 when in).
    pub lengths: Vec<u8>,
    /// Occupied cells whose piece cannot be taken, ascending.
    pub protected: Vec<Cell>,
    pub last: Option<LastMove>,
    /// Rounds of the table without progress left before the quiet limit
    /// stops the game.
    pub quiet_left: u32,
    pub end: Option<End>,
    /// The viewer's legal moves (empty when it is not its turn).
    pub legal: Vec<Action>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let lengths = s
            .turn
            .map(|seat| {
                s.pieces[seat.index()]
                    .iter()
                    .enumerate()
                    .map(|(i, spot)| match spot {
                        Spot::Finished => 0,
                        _ => move_length(s, seat, i as u8 + 1),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let protected = (0..CELLS.len() as u8)
            .filter(|&c| s.occupant(c).is_some() && is_protected(s, c))
            .collect();
        let allowed = QUIET_ROUNDS * u32::from(s.players);
        let quiet_left = allowed
            .saturating_sub(s.quiet)
            .div_ceil(u32::from(s.players));
        let legal = match viewer {
            Viewer::Seat(seat) => Bantu::legal_actions(s, seat),
            Viewer::Spectator => Vec::new(),
        };
        Self {
            players: s.players,
            pieces: s.pieces.clone(),
            turn: s.turn,
            lengths,
            protected,
            last: s.last,
            quiet_left,
            end: s.end,
            legal,
        }
    }
}
