use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State, ANTE};
use crate::deck::Rank;

/// What one seat sees: its own hand and pitched card, every stack, hand
/// sizes, who has pitched and the balances. Never another hand, a card
/// another seat has chosen or pitched, nor the draw pile. No statistics:
/// guessing what comes back is the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub hand_no: u8,
    pub hands_total: u8,
    /// The round of the hand, from 1; 0 during setup.
    pub round: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Setup: the card the viewer has chosen, not yet revealed.
    pub placed: Option<Rank>,
    /// The card the viewer has pitched this round.
    pub pitched: Option<Rank>,
    /// Seat to move during play.
    pub turn: Option<Seat>,
    pub pot: u32,
    pub ante: u32,
    /// What folding costs now.
    pub fold_cost: Option<u32>,
    pub deck: usize,
    /// Cards in the discard pile, face down ones included.
    pub discard: usize,
    /// The discards everyone has seen.
    pub discard_seen: Vec<Rank>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub stack: Vec<Rank>,
    pub hand_size: usize,
    /// Folded or paired in the current hand.
    pub out: bool,
    /// Has a card in the pool this round.
    pub pitched: bool,
    /// Balance since the start of the game.
    pub chips: i32,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            hand_no: s.hand_no,
            hands_total: s.hands_total,
            round: s.round,
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    stack: s.stacks[i].clone(),
                    hand_size: s.hands[i].len(),
                    out: s.out[i],
                    pitched: s.pitched[i].is_some(),
                    chips: s.chips[i],
                })
                .collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()]),
            pitched: me.and_then(|m| s.pitched[m.index()]),
            turn: s.turn,
            pot: s.pot,
            ante: ANTE,
            fold_cost: s.fold_cost(),
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            discard_seen: s.supply.seen().to_vec(),
            events: s.events.clone(),
        }
    }
}
