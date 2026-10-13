use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State, ANTE, PLAY_COST};
use crate::deck::Rank;

/// What one seat sees: its own hand, the street, every house, hand sizes
/// and balances. Never another hand, the draw pile, nor a card another seat
/// has chosen during setup. No statistics: weighing a draw is the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub hand_no: u8,
    pub hands_total: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    pub street: Vec<Rank>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Setup: the card the viewer has chosen, not yet revealed.
    pub placed: Option<Rank>,
    /// The seat whose house the viewer may play into besides its own.
    pub neighbour: Option<Seat>,
    /// Seat to move during play.
    pub turn: Option<Seat>,
    pub pot: u32,
    pub ante: u32,
    pub play_cost: u32,
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
    pub house: Vec<Rank>,
    pub hand_size: usize,
    /// Folded or busted in the current hand.
    pub out: bool,
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
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    house: s.houses[i].clone(),
                    hand_size: s.hands[i].len(),
                    out: s.out[i],
                    chips: s.chips[i],
                })
                .collect(),
            street: s.street.clone(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()]),
            neighbour: me
                .filter(|m| s.phase == Phase::Play && !s.out[m.index()])
                .and_then(|m| s.left_of(m)),
            turn: s.turn,
            pot: s.pot,
            ante: ANTE,
            play_cost: PLAY_COST,
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            discard_seen: s.supply.seen().to_vec(),
            events: s.events.clone(),
        }
    }
}
