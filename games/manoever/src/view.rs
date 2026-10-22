use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::board::{Square, GRAYS, LAST};
use crate::{
    bonus, destinations, find, height, max_move, position, scores, to_move, Dest, MoveRecord,
    Soldier, State,
};

/// Everything is public but the seed: the whole position, plus what the
/// board needs to show without computing it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: u8,
    /// Still bringing soldiers out with the die.
    pub setup: bool,
    /// `stacks[square - 1]`, bottom to top.
    pub stacks: Vec<Vec<Soldier>>,
    /// Soldiers of each seat still waiting to come out.
    pub waiting: Vec<u8>,
    /// `barracks[place - 1]`.
    pub barracks: Vec<Option<Soldier>>,
    pub grays: Vec<GrayView>,
    pub turn: Option<Seat>,
    /// The soldiers the player to move still has to move, with their sums.
    pub to_move: Vec<Mover>,
    pub last_roll: Option<(Seat, u8)>,
    /// Moves of the turn being played.
    pub turn_log: Vec<MoveRecord>,
    /// The last complete turn of each seat.
    pub last_turns: Vec<Vec<MoveRecord>>,
    pub scores: Vec<u32>,
    pub over: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrayView {
    pub square: Square,
    pub value: u8,
    pub taken_by: Option<Seat>,
}

/// A soldier to move: where it is and how its maximum is made.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mover {
    pub number: u8,
    pub square: Square,
    pub position: u8,
    pub height: u8,
    pub bonus: u8,
    pub max: u32,
    pub dests: Vec<Dest>,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        let to_move = match s.turn {
            Some(seat) if !s.in_setup() => to_move(s, seat)
                .into_iter()
                .filter_map(|number| {
                    let soldier = Soldier { seat, number };
                    let (square, index) = find(s, soldier)?;
                    Some(Mover {
                        number,
                        square,
                        position: position(s, square),
                        height: height(s, square, index),
                        bonus: bonus(square, index),
                        max: max_move(s, soldier),
                        dests: destinations(s, soldier),
                    })
                })
                .collect(),
            _ => Vec::new(),
        };
        Self {
            players: s.players,
            setup: s.in_setup(),
            stacks: s.stacks[1..=usize::from(LAST)].to_vec(),
            waiting: s.waiting.clone(),
            barracks: s.barracks.to_vec(),
            grays: GRAYS
                .iter()
                .zip(s.grays)
                .map(|(&(square, value), taken_by)| GrayView {
                    square,
                    value,
                    taken_by,
                })
                .collect(),
            turn: s.turn,
            to_move,
            last_roll: s.last_roll,
            turn_log: s.turn_log.clone(),
            last_turns: s.last_turns.clone(),
            scores: scores(s),
            over: s.over,
        }
    }
}
