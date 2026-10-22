use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{target, Event, Length, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its hand and the card it chose, the pool, the cards
/// revealed this turn, every pile and the scores. Never another hand,
/// another choice before the reveal, nor the deck's order. No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub deal_no: u8,
    /// Long game: the score to reach.
    pub target: Option<u16>,
    pub phase: Phase,
    pub dealer: Seat,
    pub pool: Vec<Rank>,
    pub deck_size: usize,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// The card the viewer has chosen, before the reveal.
    pub chosen: Option<Rank>,
    /// Take: who picks and the card it played.
    pub taking: Option<(Seat, Rank)>,
    pub seats: Vec<SeatView>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub pile: Vec<Rank>,
    pub pile_points: u16,
    pub hand_size: usize,
    pub score: u16,
    /// The card revealed this turn, while it has not acted yet.
    pub waiting: Option<Rank>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            deal_no: s.deal_no,
            target: (s.config.length == Length::Target).then(|| target(s.players)),
            phase: s.phase,
            dealer: s.dealer,
            pool: s.pool.clone(),
            deck_size: s.deck.len(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            chosen: me.and_then(|m| s.chosen[m.index()]),
            taking: s.taking,
            seats: (0..usize::from(s.players))
                .map(|i| {
                    let mut pile = s.piles[i].clone();
                    pile.sort_unstable();
                    SeatView {
                        pile_points: pile.iter().map(|c| u16::from(c.value())).sum(),
                        pile,
                        hand_size: s.hands[i].len(),
                        score: s.scores[i],
                        waiting: s
                            .waiting
                            .iter()
                            .find(|(seat, _)| seat.index() == i)
                            .map(|&(_, c)| c),
                    }
                })
                .collect(),
            events: s.events.clone(),
        }
    }
}
