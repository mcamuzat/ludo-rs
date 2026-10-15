use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand, the targets, the shooters and every
/// hand size. Never another hand nor the deck.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub rounds_total: u8,
    pub phase: Phase,
    pub hand_sizes: Vec<usize>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    pub targets: Vec<Rank>,
    /// This round's shooters: `shooters[seat]`, `None` until it shoots.
    pub shooters: Vec<Option<Rank>>,
    pub leader: Seat,
    pub turn: Option<Seat>,
    pub scores: Vec<u32>,
    pub events: Vec<Event>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let mut shooters = vec![None; usize::from(s.players)];
        for &(seat, rank) in &s.shooters {
            shooters[seat.index()] = Some(rank);
        }
        Self {
            round_no: s.round_no,
            rounds_total: s.rounds_total,
            phase: s.phase,
            hand_sizes: s.hands.iter().map(Vec::len).collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            targets: s.targets.clone(),
            shooters,
            leader: s.leader,
            turn: s.turn,
            scores: s.scores.clone(),
            events: s.events.clone(),
        }
    }
}
