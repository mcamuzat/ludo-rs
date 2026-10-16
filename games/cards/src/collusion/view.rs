use ludo_core::{Game, Seat, Viewer};
use serde::Serialize;

use super::{rank_value, suit_index, Action, Collusion, DealLine, Phrase, State, TakenTrick};
use crate::deck::Card;

/// What one seat sees: its own hand, every hand size, the table, the talk
/// and the scores. Never another hand nor the seed; no count of the cards
/// gone either (the user wants no statistics shown).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    /// By suit, then strongest first (empty for a spectator).
    pub hand: Vec<Card>,
    pub hand_counts: [u8; 4],
    pub trick: Vec<(Seat, Card)>,
    pub last_trick: Option<TakenTrick>,
    pub taken: [u8; 4],
    pub scores: [u32; 4],
    pub target: u32,
    pub dealer: Seat,
    /// From 1, for display.
    pub deal_no: u32,
    pub turn: Option<Seat>,
    /// Each seat's latest phrase this deal.
    pub last_said: [Option<Phrase>; 4],
    pub talk: Vec<(Seat, Phrase)>,
    pub last_deal: Option<[DealLine; 4]>,
    /// The viewer's legal actions (empty when it is not its turn).
    pub legal: Vec<Action>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < s.hands.len() => Some(seat),
            _ => None,
        };
        let mut hand = me.map(|m| s.hands[m.index()].clone()).unwrap_or_default();
        hand.sort_by_key(|c| {
            (
                c.suit().map(suit_index),
                std::cmp::Reverse(c.rank().map(rank_value)),
            )
        });
        let mut last_said = [None; 4];
        for &(seat, phrase) in &s.talk {
            last_said[seat.index()] = Some(phrase);
        }
        Self {
            me,
            hand,
            hand_counts: std::array::from_fn(|i| s.hands[i].len() as u8),
            trick: s.trick.clone(),
            last_trick: s.last_trick.clone(),
            taken: s.taken,
            scores: s.scores,
            target: s.config.target,
            dealer: s.dealer,
            deal_no: s.deal_no + 1,
            turn: s.turn,
            last_said,
            talk: s.talk.clone(),
            last_deal: s.last_deal,
            legal: me
                .map(|m| Collusion::legal_actions(s, m))
                .unwrap_or_default(),
        }
    }
}
