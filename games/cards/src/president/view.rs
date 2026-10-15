use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{hand_order, Event, Phase, State, Title, Transfer, Trick};
use crate::deck::Card;

/// What one seat sees: its own hand and its part of the exchange, every
/// hand size, the table and the scores. Never another hand, the seed, nor
/// an exchange it took no part in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub rounds_total: u8,
    pub phase: Phase,
    pub reversed: bool,
    pub hand_sizes: Vec<usize>,
    /// The viewer's hand, weakest first under the order in force (empty for
    /// a spectator).
    pub hand: Vec<Card>,
    pub trick: Option<Trick>,
    /// Who passed since the last play.
    pub passes: Vec<Seat>,
    /// Play: whose turn it is.
    pub turn: Option<Seat>,
    /// Titles from the previous round (empty during the first).
    pub titles: Vec<Title>,
    pub scores: Vec<u32>,
    /// This round: who went out, and who went out on a forbidden card.
    pub out: Vec<Seat>,
    pub fouled: Vec<Seat>,
    pub set_aside: Vec<Card>,
    /// This round's exchange, as far as the viewer took part in it.
    pub received: Vec<Card>,
    pub given: Vec<Card>,
    /// Exchange: who still has to hand cards back.
    pub to_give: Vec<Seat>,
    pub events: Vec<Event>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let mut hand = me.map(|m| s.hands[m.index()].clone()).unwrap_or_default();
        hand.sort_by_key(|&c| hand_order(c, s.reversed));
        let mine = |side: fn(&Transfer) -> Seat| -> Vec<Card> {
            me.map(|m| {
                s.transfers
                    .iter()
                    .filter(|t| side(t) == m)
                    .flat_map(|t| t.cards.iter().copied())
                    .collect()
            })
            .unwrap_or_default()
        };
        Self {
            round_no: s.round_no,
            rounds_total: s.config.rounds,
            phase: s.phase,
            reversed: s.reversed,
            hand_sizes: s.hands.iter().map(Vec::len).collect(),
            hand,
            trick: s.trick.clone(),
            passes: s.passes.clone(),
            turn: if s.phase == Phase::Play { s.turn } else { None },
            titles: s.titles.clone(),
            scores: s.scores.clone(),
            out: s.out.clone(),
            fouled: s.fouled.clone(),
            set_aside: s.set_aside.clone(),
            received: mine(|t| t.to),
            given: mine(|t| t.from),
            to_give: s.to_give.clone(),
            events: s.events.clone(),
        }
    }
}
