use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Order, Phase, RoundResult, State};
use crate::deck::Rank;

/// What one seat sees: its own hand and face-down discards, hand sizes,
/// the Order, rounds won, and this round's events with only the cards it
/// gave, received or discarded. Never another hand nor the undealt cards.
/// No statistics: reading the others is the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round: u8,
    pub wins_needed: u8,
    pub dealer: Seat,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// The viewer's face-down discards this round.
    pub discarded: Vec<Rank>,
    pub order: Option<Order>,
    pub turn: Option<Seat>,
    pub log: Vec<Event>,
    pub last_round: Option<RoundResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub hand_size: usize,
    pub wins: u8,
}

/// `card` if `viewer` is one of `knowers`.
fn masked(card: Option<Rank>, viewer: Option<Seat>, knowers: &[Seat]) -> Option<Rank> {
    card.filter(|_| viewer.is_some_and(|v| knowers.contains(&v)))
}

fn mask(event: &Event, viewer: Option<Seat>) -> Event {
    match *event {
        Event::Followed { seat, owner, card } => Event::Followed {
            seat,
            owner,
            card: masked(card, viewer, &[seat, owner]),
        },
        Event::Passed { from, to, card } => Event::Passed {
            from,
            to,
            card: masked(card, viewer, &[from, to]),
        },
        Event::Gave { from, to, card } => Event::Gave {
            from,
            to,
            card: masked(card, viewer, &[from, to]),
        },
        Event::Discarded { seat, card } => Event::Discarded {
            seat,
            card: masked(card, viewer, &[seat]),
        },
        Event::Ordered { .. } | Event::OrderDone { .. } | Event::Asked { .. } => event.clone(),
    }
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            round: s.round,
            wins_needed: s.wins_needed,
            dealer: s.dealer,
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    hand_size: s.hands[i].len(),
                    wins: s.wins[i],
                })
                .collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            discarded: me
                .map(|m| s.discarded[m.index()].clone())
                .unwrap_or_default(),
            order: s.order,
            turn: s.turn,
            log: s.log.iter().map(|e| mask(e, me)).collect(),
            last_round: s.last_round.clone(),
        }
    }
}
