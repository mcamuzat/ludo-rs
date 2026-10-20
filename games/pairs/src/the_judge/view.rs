use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{recipe_points, target, Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand, every recipe, who has passed, and the
/// scores. Never another hand, the Judge's cards before the contest, nor
/// the cards left undealt. No statistics: reading the others is the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub target: u16,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Cook: the hand cards the viewer may add now.
    pub addable: Vec<Rank>,
    /// Face-down cards in the Judge.
    pub judge_size: usize,
    pub leader: Seat,
    pub turn: Option<Seat>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub recipe: Vec<Rank>,
    /// What the recipe would score.
    pub recipe_points: u16,
    pub hand_size: usize,
    pub passed: bool,
    pub score: u16,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let cooking = s.phase == Phase::Cook;
        Self {
            round_no: s.round_no,
            target: target(s.players),
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    recipe: s.recipes[i].clone(),
                    recipe_points: recipe_points(&s.recipes[i]),
                    hand_size: s.hands[i].len(),
                    passed: s.passed[i],
                    score: s.scores[i],
                })
                .collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            addable: me
                .filter(|&m| cooking && s.turn == m)
                .map(|m| s.addable(m))
                .unwrap_or_default(),
            judge_size: s.judge.len(),
            leader: s.leader,
            turn: cooking.then_some(s.turn),
            events: s.events.clone(),
        }
    }
}
