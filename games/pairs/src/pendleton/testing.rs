//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, DEFAULT_TARGET};
use crate::deck::Supply;
use crate::testing::{r, ranks};

/// Round 1: these targets and sides (`sides[seat][column]`), `seat` to
/// play `drawn`. `deck` is drawn from its end.
pub fn table(targets: &[u8], sides: [&[&[u8]]; 2], drawn: u8, seat: u8, deck: &[u8]) -> State {
    State {
        target_score: DEFAULT_TARGET,
        round_no: 1,
        seed: 0,
        supply: Supply::from_parts(ranks(deck), Vec::new(), Vec::new(), 0, 0),
        targets: ranks(targets),
        played: sides
            .iter()
            .map(|side| {
                let mut columns: Vec<_> = side.iter().map(|c| ranks(c)).collect();
                columns.resize(targets.len(), Vec::new());
                columns
            })
            .collect(),
        drawn: Some(r(drawn)),
        turn: Some(Seat(seat)),
        first: Seat(0),
        phase: Phase::Play,
        scores: vec![0, 0],
        events: Vec::new(),
    }
}
