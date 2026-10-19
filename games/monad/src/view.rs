use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::{monads_to_win, Action, Card, Color, Event, Rank, State, Turn};

/// One move of a turn, as `viewer` may see it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventView {
    pub seat: Seat,
    pub action: Action,
    pub gained: Vec<Card>,
    pub monad: bool,
    /// A Unit was taken from the face-down deck.
    pub drew: bool,
    /// Which one: for the seat that took it only.
    pub drawn: Option<Color>,
}

impl EventView {
    fn of(event: &Event, viewer: Viewer) -> Self {
        EventView {
            seat: event.seat,
            action: event.action.clone(),
            gained: event.gained.clone(),
            monad: event.monad,
            drew: event.drawn.is_some(),
            drawn: event.drawn.filter(|_| viewer == Viewer::Seat(event.seat)),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: u8,
    /// Monads needed to win.
    pub target: u8,
    pub identities: Vec<Color>,
    pub monads: Vec<u8>,
    pub monad_stock: u8,
    /// Bi, Tri, Quad, Quint; top card first.
    pub columns: Vec<Vec<Color>>,
    /// Bottom first.
    pub discard: Vec<Color>,
    pub deck: u32,
    /// The viewer's own hand; `None` for a spectator.
    pub hand: Option<Vec<Card>>,
    /// Cards held by each seat, by rank (Unit … Quint).
    pub hand_counts: Vec<[u8; 5]>,
    /// `None` once the game is over.
    pub current: Option<Seat>,
    pub turn: Turn,
    pub this_turn: Vec<EventView>,
    pub last_turn: Vec<EventView>,
    pub winner: Option<Seat>,
    pub stalled: bool,
    /// Final ranks (0 = first), once the game is over.
    pub ranks: Option<Vec<u8>>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let over = s.over();
        let events = |events: &[Event]| events.iter().map(|e| EventView::of(e, viewer)).collect();
        View {
            players: s.players,
            target: monads_to_win(s.players),
            identities: s.identities.clone(),
            monads: s.monads.clone(),
            monad_stock: s.monad_stock,
            columns: s
                .columns
                .iter()
                .map(|c| c.iter().copied().collect())
                .collect(),
            discard: s.discard.clone(),
            deck: u32::try_from(s.deck.len()).expect("at most 24 Units"),
            hand: match viewer {
                Viewer::Seat(seat) => Some(s.hands[seat.index()].clone()),
                Viewer::Spectator => None,
            },
            hand_counts: s
                .hands
                .iter()
                .map(|hand| {
                    Rank::ALL.map(|rank| {
                        let n = hand.iter().filter(|c| c.rank == rank).count();
                        u8::try_from(n).expect("at most 24 cards of a rank")
                    })
                })
                .collect(),
            current: (!over).then_some(s.current),
            turn: s.turn.clone(),
            this_turn: events(&s.this_turn),
            last_turn: events(&s.last_turn),
            winner: s.winner,
            stalled: s.stalled,
            ranks: over.then(|| s.ranks()),
        }
    }
}
