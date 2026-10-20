use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{card_points, Event, Phase, State, TARGET};
use crate::deck::Rank;

/// What one seat sees: its own face-down cards, every face-up hand, the
/// declaration on the table, and the scores. Never another face-down card,
/// the played card before it is revealed (except to its player), nor the
/// cards left undealt. No statistics: reading the others is the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub target: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's face-down cards (empty for a spectator).
    pub hand: Vec<Rank>,
    pub table: Option<TableView>,
    /// Declare: who plays the next card.
    pub turn: Option<Seat>,
    /// Place: who decides where the card goes.
    pub chooser: Option<Seat>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub shown: Vec<Rank>,
    /// The ranks of the face-up hand added up.
    pub card_points: u32,
    pub hand_size: usize,
    pub score: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableView {
    pub poser: Seat,
    pub declared: Rank,
    /// Once revealed, or to its player.
    pub card: Option<Rank>,
    /// Answer: who must answer now.
    pub asked: Option<Seat>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            round_no: s.round_no,
            target: TARGET,
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    shown: s.shown[i].clone(),
                    card_points: card_points(&s.shown[i]),
                    hand_size: s.hands[i].len(),
                    score: s.scores[i],
                })
                .collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            table: s.table.as_ref().map(|t| TableView {
                poser: t.poser,
                declared: t.declared,
                card: (s.phase == Phase::Place || me == Some(t.poser)).then_some(t.card),
                asked: (s.phase == Phase::Answer).then_some(t.asked),
            }),
            turn: (s.phase == Phase::Declare).then_some(s.turn),
            chooser: s.chooser,
            events: s.events.clone(),
        }
    }
}
