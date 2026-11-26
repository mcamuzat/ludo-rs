use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::{
    final_points, Color, Conflict, EndReason, Event, LastTurn, Phase, Slot, State, QUIET_ROUNDS,
};

/// A seat's points at the end.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalScore {
    /// Points by colour (black, red, blue, green), treasures apart.
    pub scores: [u8; 4],
    pub treasures: u8,
    /// The points once each treasure is given to the weakest colour.
    pub with_treasures: [u32; 4],
    /// The weakest colour's points: the score.
    pub weakest: u32,
    pub rank: u8,
}

/// What a decision is about, for the panel under the table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all_fields = "camelCase")]
pub enum PhaseView {
    Action,
    Conflict {
        conflict: Conflict,
        /// The deciding seat's cards of the support colour: for it alone.
        most: Option<u8>,
    },
    ChooseConflict {
        colors: Vec<Color>,
    },
    Ship {
        column: u8,
        options: Vec<Color>,
    },
    Over,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    pub players: u8,
    pub heads: Vec<Slot>,
    pub gaps: Vec<Option<Slot>>,
    pub columns: Vec<Vec<Slot>>,
    /// The junction still face down while its conflicts last.
    pub join: Option<u8>,
    /// My cards by colour.
    pub hand: Option<[u8; 4]>,
    pub hand_sizes: Vec<u8>,
    /// Leaders not on the table, by seat.
    pub supply: Vec<Vec<Color>>,
    pub catastrophes: Vec<bool>,
    /// Score piles by colour, visible to all.
    pub scores: Vec<[u8; 4]>,
    pub treasures: Vec<u8>,
    /// Ships still to build.
    pub ships: Vec<Color>,
    pub deck: u32,
    pub treasures_left: u8,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    pub deciding: Option<Seat>,
    pub actions_left: u8,
    pub phase: PhaseView,
    /// Turns without a draw before the site ends the game.
    pub quiet_left: u32,
    pub steps: Vec<Event>,
    pub last_turn: Option<LastTurn>,
    pub end: Option<EndReason>,
    pub final_scores: Option<Vec<FinalScore>>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let n = usize::from(s.players);
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < n => Some(seat),
            _ => None,
        };
        let phase = match &s.phase {
            Phase::Action => PhaseView::Action,
            Phase::Conflict(c) => PhaseView::Conflict {
                conflict: *c,
                most: (me == Some(c.deciding()))
                    .then(|| s.hands[c.deciding().index()][c.support().index()]),
            },
            Phase::ChooseConflict(colors) => PhaseView::ChooseConflict {
                colors: colors.clone(),
            },
            Phase::Ship { column, options } => PhaseView::Ship {
                column: *column,
                options: options.clone(),
            },
            Phase::Over => PhaseView::Over,
        };
        let over = s.phase == Phase::Over;
        let final_scores = over.then(|| {
            let ranks = s.ranks();
            (0..n)
                .map(|i| {
                    let with_treasures = final_points(&s.scores[i], s.treasures[i]);
                    FinalScore {
                        scores: s.scores[i],
                        treasures: s.treasures[i],
                        with_treasures,
                        weakest: *with_treasures.iter().min().expect("four colours"),
                        rank: ranks[i],
                    }
                })
                .collect()
        });
        Self {
            me,
            players: s.players,
            heads: s.table.heads.clone(),
            gaps: s.table.gaps.clone(),
            columns: s.table.columns.clone(),
            join: s.join,
            hand: me.map(|m| s.hands[m.index()]),
            hand_sizes: s.hands.iter().map(|h| h.iter().sum()).collect(),
            supply: (0..s.players).map(|i| s.supply(Seat(i))).collect(),
            catastrophes: s.catastrophes.clone(),
            scores: s.scores.clone(),
            treasures: s.treasures.clone(),
            ships: s.ships.clone(),
            deck: s.deck.len() as u32,
            treasures_left: s.treasures_left(),
            turn: (!over).then_some(s.turn),
            deciding: s.deciding(),
            actions_left: s.actions_left,
            phase,
            quiet_left: (QUIET_ROUNDS * u32::from(s.players)).saturating_sub(s.quiet),
            steps: s.steps.clone(),
            last_turn: s.last_turn.clone(),
            end: s.end,
            final_scores,
        }
    }
}
