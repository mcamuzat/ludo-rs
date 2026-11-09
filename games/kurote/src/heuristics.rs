use ludo_core::{Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{Action, Kurote, State};

/// How many opposing wizards `action` captures.
fn gain(s: &State, seat: Seat, action: Action) -> usize {
    let mut after = s.clone();
    after.play(seat, action);
    after.last.map_or(0, |l| l.captured.len())
}

/// Among `moves`, the one that captures the most, if any captures.
pub(crate) fn best_capture(s: &State, seat: Seat, moves: &[Action]) -> Option<Action> {
    moves
        .iter()
        .map(|&a| (gain(s, seat, a), a))
        .filter(|&(g, _)| g > 0)
        .max_by_key(|&(g, _)| g)
        .map(|(_, a)| a)
}

impl Heuristics for Kurote {
    /// Capture when possible, else play at random.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let moves = s.moves(seat);
        best_capture(s, seat, &moves)
            .unwrap_or_else(|| *moves.choose(rng).expect("a pending seat has an action"))
    }
}
