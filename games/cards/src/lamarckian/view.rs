use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{best_hand, lowest_rank, sorted, Event, Phase, PokerHand, ShowdownLine, State, TURNS};
use crate::deck::{Card, Rank};

/// What one seat sees: its hand, its best poker hand and the mark it chose,
/// the pool, this turn's marks, the cards each player was seen to take, and
/// the scores. Never another hand's unseen cards, another mark before the
/// reveal, nor the deck's order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    pub competition: u8,
    pub competitions: u8,
    pub turn_no: u8,
    pub turns: u8,
    /// The lowest rank in the pack.
    pub lowest: Rank,
    pub phase: Phase,
    pub pool: Vec<Card>,
    pub deck_size: usize,
    /// The viewer's hand, by suit (empty for a spectator).
    pub hand: Vec<Card>,
    /// The best poker hand the viewer holds now.
    pub best: Option<PokerHand>,
    /// The mark the viewer has chosen, before the reveal.
    pub chosen: Option<Card>,
    /// Pick: who picks and its mark.
    pub picking: Option<(Seat, Card)>,
    pub seats: Vec<SeatView>,
    pub last_showdown: Option<Vec<ShowdownLine>>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub hand_size: usize,
    /// The cards everyone saw this player take, by suit.
    pub known: Vec<Card>,
    pub score: u32,
    /// This turn's mark, once revealed.
    pub mark: Option<Card>,
    /// The mark has not acted yet.
    pub waiting: bool,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let hand = me
            .map(|m| sorted(s.hands[m.index()].clone()))
            .unwrap_or_default();
        Self {
            me,
            competition: s.competition,
            competitions: s.config.competitions,
            turn_no: s.turn_no,
            turns: TURNS,
            lowest: lowest_rank(s.players),
            phase: s.phase,
            pool: s.pool.clone(),
            deck_size: s.deck.len(),
            best: best_hand(&hand),
            hand,
            chosen: me.and_then(|m| s.chosen[m.index()]),
            picking: s.picking,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    hand_size: s.hands[i].len(),
                    known: sorted(s.known[i].clone()),
                    score: s.scores[i],
                    mark: s.revealed[i],
                    waiting: s.waiting.iter().any(|(seat, _)| seat.index() == i)
                        || s.picking.is_some_and(|(seat, _)| seat.index() == i),
                })
                .collect(),
            last_showdown: s.last_showdown.clone(),
            events: s.events.clone(),
        }
    }
}
