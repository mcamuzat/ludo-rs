use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{Action, Pennywise, State};

impl Heuristics for Pennywise {
    /// Any legal play: the tree search does the thinking.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        Self::legal_actions(s, seat)
            .choose(rng)
            .cloned()
            .expect("rollout for the seat whose turn it is")
    }
}
