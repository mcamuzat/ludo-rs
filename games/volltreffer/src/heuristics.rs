use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{publication_points, Action, Phase, State, Volltreffer};

impl Heuristics for Volltreffer {
    /// Hire any card the seat can pay for; show what scores most unless it
    /// overshoots the Sanctuary; play any card that may be played.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if s.phase == Phase::Publish {
            let hand = &s.hands[seat.index()];
            let score = s.scores[seat.index()];
            let top = *s.config.sanctuary().end();
            let points = |a: &Action| match a {
                Action::Publish { groups, tribes } => publication_points(hand, groups, *tribes),
                _ => 0,
            };
            return legal
                .iter()
                .filter(|a| score + points(a) <= top)
                .max_by_key(|a| points(a))
                .unwrap_or(&legal[0])
                .clone();
        }
        legal
            .choose(rng)
            .expect("a pending seat has a legal action")
            .clone()
    }

    /// A simulation stops once the round it started in is over: the next
    /// round is chance. The standing is then the end-of-game one.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.phase == Phase::Over || sim.round > root.round)
            .then(|| Outcome::Ranked(sim.ranks()).rewards())
    }
}
