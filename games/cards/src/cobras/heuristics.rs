use ludo_core::{Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Cobras, Phase, State};
use crate::standings;

impl Heuristics for Cobras {
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        *s.moves(seat)
            .choose(rng)
            .expect("a pending seat has an action")
    }

    /// A simulation stops once the round it started in is scored: the next
    /// deal is chance.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.round > root.round || sim.phase == Phase::Over)
            .then(|| Outcome::Ranked(standings(&sim.scores)).rewards())
    }
}
