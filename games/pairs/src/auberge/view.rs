use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand, everyone's face-up cards and hand
/// sizes. Never another hand, the draw pile, nor a card another seat has
/// chosen during setup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round: u8,
    pub rounds: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Setup: the card the viewer has chosen, not yet revealed.
    pub placed: Option<Rank>,
    /// The pair the viewer holds, if any: an accusation would eliminate it.
    pub own_pair: Option<Rank>,
    /// Seat to move during play.
    pub turn: Option<Seat>,
    pub deck: usize,
    /// Cards in the discard pile, face down ones included.
    pub discard: usize,
    /// The discards everyone has seen.
    pub discard_seen: Vec<Rank>,
    /// Per rank (index = rank − 1), the cards the viewer cannot place.
    pub unlocated: Vec<u8>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub front: Vec<Rank>,
    pub hand_size: usize,
    /// Eliminated from the current round.
    pub out: bool,
    /// Rounds won.
    pub wins: u8,
    /// The viewer's estimate, per thousand, that this seat holds a pair
    /// (0 for the viewer itself and for an eliminated seat).
    pub pair_odds: u16,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let seats = (0..s.players)
            .map(Seat)
            .map(|seat| {
                let i = seat.index();
                let hidden = Some(seat) == me || s.out[i];
                SeatView {
                    front: s.fronts[i].clone(),
                    hand_size: s.hands[i].len(),
                    out: s.out[i],
                    wins: s.wins[i],
                    pair_odds: if hidden {
                        0
                    } else {
                        (s.pair_chance(me, seat) * 1000.0).round() as u16
                    },
                }
            })
            .collect();
        let events = s
            .events
            .iter()
            .cloned()
            .map(|event| match event {
                // ❓6: the hand is shown to the accuser alone (the accused
                // knows its own).
                Event::Accused {
                    seat,
                    target,
                    guilty,
                    hand,
                } => Event::Accused {
                    seat,
                    target,
                    guilty,
                    hand: if me == Some(seat) || me == Some(target) {
                        hand
                    } else {
                        Vec::new()
                    },
                },
                other => other,
            })
            .collect();
        Self {
            round: s.round,
            rounds: s.rounds,
            phase: s.phase,
            seats,
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()]),
            own_pair: me.and_then(|m| s.own_pair(m)),
            turn: s.turn,
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            discard_seen: s.supply.seen().to_vec(),
            unlocated: s.unlocated_for(me).to_vec(),
            events,
        }
    }
}
