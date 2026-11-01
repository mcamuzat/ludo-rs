use ludo_core::{Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{Action, BlackHole, State};

impl Heuristics for BlackHole {
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        *s.moves(seat)
            .choose(rng)
            .expect("a pending seat has a move")
    }
}
