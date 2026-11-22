use ludo_core::ai::{Budget, Level};
use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use crate::{Action, FiveByFifteen, State};

/// A solo game: no AI seat plays it, but the platform wants a policy.
impl Heuristics for FiveByFifteen {
    fn budget(_level: Level, _timed: bool) -> Option<Budget> {
        None
    }

    /// Any move but giving up, until a game drags on.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let moves: Vec<Action> = Self::legal_actions(s, seat)
            .into_iter()
            .filter(|a| *a != Action::GiveUp)
            .collect();
        if moves.is_empty() || s.moves >= 400 {
            return Action::GiveUp;
        }
        moves[rng.gen_range(0..moves.len() as u32) as usize].clone()
    }
}
