use ludo_core::Seat;
use serde::Serialize;

use super::{Event, State};
use crate::deck::Rank;

/// Stacks and scores are public and the draw pile is hidden from everyone:
/// every viewer gets the same view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub seats: Vec<SeatView>,
    /// The losing score.
    pub target: u8,
    /// Seat to move; `None` once the game is over.
    pub turn: Option<Seat>,
    pub loser: Option<Seat>,
    /// Cards in the draw pile.
    pub deck: usize,
    /// Cards in the discard pile, burned cards included.
    pub discard: usize,
    /// Per rank (index = rank − 1), the cards nobody can place: the draw
    /// pile and the burned cards.
    pub unlocated: Vec<u8>,
    /// The lowest card in play: what the cheapest fold costs.
    pub lowest: Option<Rank>,
    /// What happened since the previous decision.
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub stack: Vec<Rank>,
    pub scored: Vec<Rank>,
    pub score: u8,
}

impl View {
    pub fn of(s: &State) -> Self {
        Self {
            seats: (0..s.players)
                .map(Seat)
                .map(|seat| SeatView {
                    stack: s.stacks[seat.index()].clone(),
                    scored: s.scored[seat.index()].clone(),
                    score: s.score(seat),
                })
                .collect(),
            target: s.target,
            turn: s.turn,
            loser: s.loser,
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            unlocated: s.supply.unlocated().to_vec(),
            lowest: s.lowest_in_play(),
            events: s.events.clone(),
        }
    }
}
