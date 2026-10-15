use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand and discards, how many cards each seat
/// holds and discarded, who is still in, the pot and the balances. Never
/// another hand, another seat's discards, nor the undealt cards.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub game_no: u8,
    pub games_total: u8,
    pub round_no: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// What the viewer gave the Monster this round.
    pub discarded: Vec<Rank>,
    /// Cards in the Monster's hand.
    pub monster_size: usize,
    pub button: Seat,
    pub turn: Option<Seat>,
    pub pot: u32,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    /// Still in this game.
    pub in_game: bool,
    /// Dealt into this round.
    pub in_round: bool,
    pub hand_size: usize,
    /// Has acted this round: `discards` cards, or folded.
    pub acted: bool,
    pub folded: bool,
    pub discards: usize,
    /// Balance since the start of the match.
    pub chips: i32,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        Self {
            game_no: s.game_no,
            games_total: s.games_total,
            round_no: s.round_no,
            phase: s.phase,
            seats: (0..usize::from(s.players))
                .map(|i| SeatView {
                    in_game: s.in_game[i],
                    in_round: s.in_round[i],
                    hand_size: s.hands[i].len(),
                    acted: s.acted[i],
                    folded: !s.folded[i].is_empty(),
                    discards: s.discards[i].len(),
                    chips: s.chips[i],
                })
                .collect(),
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            discarded: me
                .map(|m| s.discards[m.index()].clone())
                .unwrap_or_default(),
            monster_size: s.discards.iter().map(Vec::len).sum(),
            button: s.button,
            turn: s.turn,
            pot: s.pot,
            events: s.events.clone(),
        }
    }
}
