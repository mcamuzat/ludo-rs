use ludo_core::{Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::board::{is_corner, is_river, CELLS};
use crate::{
    final_points, Color, Conflict, EndReason, Event, Phase, Piece, Reinforcements, State, COLORS,
    MONUMENTS, QUIET_ROUNDS,
};

/// The events of a finished turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastTurn {
    pub seat: Seat,
    pub steps: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellView {
    pub river: bool,
    pub piece: Option<Piece>,
    pub treasure: bool,
    /// One of the four corner treasures' squares.
    pub corner: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonumentView {
    pub colors: [Color; 2],
    /// Top-left cell, once built.
    pub at: Option<u8>,
}

/// A seat's points at the end, screens lifted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalScore {
    /// Points by colour (black, red, blue, green), treasures apart.
    pub scores: [u32; 4],
    pub treasures: u32,
    /// The points once each treasure is given to the weakest colour.
    pub with_treasures: [u32; 4],
    /// The weakest colour's points: the score.
    pub weakest: u32,
    pub rank: u8,
}

/// What a decision is about, for the panel under the board.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all_fields = "camelCase")]
pub enum PhaseView {
    Action,
    Reinforce {
        conflict: Conflict,
        /// What the deciding seat may add.
        options: Vec<u8>,
    },
    ChooseConflict {
        colors: Vec<Color>,
    },
    Monument {
        squares: Vec<u8>,
    },
    Treasure {
        seat: Seat,
        choices: Vec<u8>,
    },
    Over,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    pub players: u8,
    pub reinforcements: Reinforcements,
    /// Row by row from the top, 16 per row.
    pub cells: Vec<CellView>,
    pub monuments: Vec<MonumentView>,
    pub junction: Option<u8>,
    /// My tiles by colour (black, red, blue, green).
    pub hand: Option<[u8; 4]>,
    pub hand_sizes: Vec<u8>,
    /// Each seat's leaders not on the board.
    pub supply: Vec<Vec<Color>>,
    pub catastrophes: Vec<u8>,
    /// My points by colour; the others' stay behind their screens.
    pub my_scores: Option<[u32; 4]>,
    /// Treasures each seat took (seen at the table).
    pub treasures: Vec<u32>,
    pub bag: u32,
    pub treasures_left: u8,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    pub actions_left: u8,
    pub deciding: Option<Seat>,
    pub phase: PhaseView,
    pub steps: Vec<Event>,
    pub last_turn: Option<LastTurn>,
    /// Rounds of the table without progress left before the game ends.
    pub quiet_left: u32,
    pub end: Option<EndReason>,
    pub final_scores: Option<Vec<FinalScore>>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.0 < s.players => Some(seat),
            _ => None,
        };
        let players = u32::from(s.players);
        let ranks = s.ranks();
        let final_scores = s.end.map(|_| {
            (0..usize::from(s.players))
                .map(|i| {
                    let with = final_points(&s.scores[i], s.treasures[i]);
                    FinalScore {
                        scores: s.scores[i],
                        treasures: s.treasures[i],
                        with_treasures: with,
                        weakest: *with.iter().min().expect("four colours"),
                        rank: ranks[i],
                    }
                })
                .collect()
        });
        Self {
            me,
            players: s.players,
            reinforcements: s.config.reinforcements,
            cells: (0..CELLS)
                .map(|c| CellView {
                    river: is_river(c as u8),
                    piece: s.cells[c],
                    treasure: s.treasure[c],
                    corner: is_corner(c as u8),
                })
                .collect(),
            monuments: MONUMENTS
                .iter()
                .zip(&s.monuments)
                .map(|(&colors, &at)| MonumentView { colors, at })
                .collect(),
            junction: s.junction,
            hand: me.map(|m| s.hands[m.index()]),
            hand_sizes: s.hands.iter().map(|h| h.iter().sum()).collect(),
            supply: (0..s.players)
                .map(|i| {
                    COLORS
                        .into_iter()
                        .filter(|&c| s.leader_at(Seat(i), c).is_none())
                        .collect()
                })
                .collect(),
            catastrophes: s.catastrophes.clone(),
            my_scores: me.map(|m| s.scores[m.index()]),
            treasures: s.treasures.clone(),
            bag: s.bag.len() as u32,
            treasures_left: s.treasures_left(),
            turn: s.end.is_none().then_some(s.turn),
            actions_left: s.actions_left,
            deciding: s.deciding(),
            phase: match &s.phase {
                Phase::Action => PhaseView::Action,
                Phase::Reinforce(c) => PhaseView::Reinforce {
                    conflict: *c,
                    options: if me == Some(c.deciding()) {
                        s.reinforce_options(c)
                    } else {
                        Vec::new()
                    },
                },
                Phase::ChooseConflict(colors) => PhaseView::ChooseConflict {
                    colors: colors.clone(),
                },
                Phase::Monument(squares) => PhaseView::Monument {
                    squares: squares.clone(),
                },
                Phase::Treasure { seat, choices } => PhaseView::Treasure {
                    seat: *seat,
                    choices: choices.clone(),
                },
                Phase::Over => PhaseView::Over,
            },
            steps: s.steps.clone(),
            last_turn: s.last_turn.clone(),
            quiet_left: (QUIET_ROUNDS * players)
                .saturating_sub(s.quiet)
                .div_ceil(players),
            end: s.end,
            final_scores,
        }
    }
}
