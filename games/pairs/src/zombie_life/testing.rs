//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, Status, DEFAULT_TARGET};
use crate::deck::Supply;
use crate::testing::ranks;

fn supply(draws: &[u8]) -> Supply {
    let mut deck = ranks(draws);
    deck.reverse();
    Supply::from_parts(deck, Vec::new(), Vec::new(), 0, 0)
}

/// Round 1 in play: these refuges and horde, everyone in, seat 0 to move.
/// `draws` are the next cards of the draw pile, in order.
pub fn table(refuges: &[&[u8]], horde: &[u8], draws: &[u8]) -> State {
    let n = refuges.len();
    State {
        players: n as u8,
        target: DEFAULT_TARGET,
        round: 1,
        supply: supply(draws),
        seed: 0,
        refuges: refuges.iter().map(|r| ranks(r)).collect(),
        horde: ranks(horde),
        status: vec![Status::In; n],
        hidden: vec![0; n],
        eliminated: Vec::new(),
        scores: vec![0; n],
        phase: Phase::Play,
        turn: Some(Seat(0)),
        drawn: None,
        dealing: Vec::new(),
        events: Vec::new(),
    }
}

/// The start of round 2: `dealer` holds `cards` and has given none yet.
pub fn dealing(scores: &[u16], dealer: u8, cards: &[u8], draws: &[u8]) -> State {
    let n = scores.len();
    let mut s = table(&vec![&[][..]; n], &[], draws);
    s.round = 2;
    s.scores = scores.to_vec();
    s.phase = Phase::Dealing;
    s.turn = Some(Seat(dealer));
    s.dealing = ranks(cards);
    s
}
