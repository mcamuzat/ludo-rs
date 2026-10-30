use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use crate::board::CellId;
use crate::{Action, SiberianDice, State};

/// Placeholder until the AI task: a random legal action.
impl Heuristics for SiberianDice {
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        legal[rng.gen::<u32>() as usize % legal.len()]
    }
}

pub fn gain(_s: &State, _seat: Seat, _to: CellId) -> i32 {
    0
}
