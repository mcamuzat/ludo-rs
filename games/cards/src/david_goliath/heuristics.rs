use ludo_core::{Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{hand_points, Action, DavidGoliath, State};
use crate::standings;

impl Heuristics for DavidGoliath {
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        *s.moves(seat)
            .choose(rng)
            .expect("a pending seat has a card to play")
    }

    /// A simulation stops once the round it started in is scored: the next
    /// deal is chance.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.round > root.round || sim.turn.is_none())
            .then(|| Outcome::Ranked(standings(&sim.scores)).rewards())
    }
}

/// Points of the cards each seat has won so far this round.
pub(super) fn current_points(s: &State) -> Vec<i32> {
    s.won.iter().map(|w| hand_points(w)).collect()
}
