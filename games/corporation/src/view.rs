use ludo_core::{Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::{room_color, Action, Color, End, Phase, State, Step, COLORS, POSITIONS, QUIET_ROUNDS};

/// The steps of a finished turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastTurn {
    pub seat: Seat,
    pub steps: Vec<Step>,
}

/// Nothing is hidden: every seat sees the whole game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    /// Each seat's colour; the colours past them are neutral.
    pub colors: Vec<Color>,
    pub table: Vec<Option<Color>>,
    /// Counters left to lay while setting up, by colour (red, blue, green,
    /// yellow).
    pub supply: Vec<u8>,
    /// Each position's colour (desks, offices; none for the boardroom).
    pub rooms: Vec<Option<Color>>,
    /// Each position's stack, bottom first.
    pub stacks: Vec<Vec<Color>>,
    /// Employees not yet entered, by colour.
    pub outside: Vec<u8>,
    /// Employees in the boardroom, by colour.
    pub arrived: Vec<u8>,
    pub phase: Phase,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// The positions whose top employee a promotion applies to.
    pub eligible: Vec<u8>,
    /// The steps of the turn being played.
    pub steps: Vec<Step>,
    /// The last finished turn (the winning one included).
    pub last_turn: Option<crate::LastTurn>,
    /// Rounds of the table without an entry or a promotion left before the
    /// game is drawn.
    pub quiet_left: u32,
    pub end: Option<End>,
    pub legal: Vec<Action>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.0 < s.players => Some(seat),
            _ => None,
        };
        let players = u32::from(s.players);
        Self {
            me,
            colors: COLORS[..usize::from(s.players)].to_vec(),
            table: s.table.clone(),
            supply: COLORS.iter().map(|&c| s.supply(c)).collect(),
            rooms: (0..POSITIONS as u8).map(room_color).collect(),
            stacks: s.stacks.clone(),
            outside: s.outside.to_vec(),
            arrived: COLORS.iter().map(|&c| s.arrived(c)).collect(),
            phase: s.phase,
            turn: s.end.is_none().then_some(s.turn),
            eligible: s.eligible(),
            steps: s.steps.clone(),
            last_turn: s.last_turn.clone(),
            quiet_left: (QUIET_ROUNDS * players)
                .saturating_sub(s.quiet)
                .div_ceil(players),
            end: s.end,
            legal: me.map(|m| s.moves(m)).unwrap_or_default(),
        }
    }
}
