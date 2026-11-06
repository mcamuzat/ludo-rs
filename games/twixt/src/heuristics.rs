//! The AI: twixtbot's network. Easy draws from the policy, Medium plays its
//! best move, Hard searches (`mcts`). The first move and the swap follow
//! twixtbot's swap model.

use ludo_core::ai::{Budget, Level, Search};
use ludo_core::{Heuristics, Seat};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use crate::mcts::{mcts, policy};
use crate::net::{net, Net};
use crate::swap::{first_move, swap_wanted};
use crate::{Twixt, TwixtAction, TwixtState};

/// Medium's move: the policy's favourite hole.
pub fn best_policy_move(net: &Net, s: &TwixtState) -> Option<TwixtAction> {
    policy(net, &s.board)
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(p, _)| TwixtAction::Place(p))
}

impl Heuristics for Twixt {
    fn rollout_action<R: Rng + ?Sized>(s: &TwixtState, _seat: Seat, rng: &mut R) -> TwixtAction {
        s.actions()
            .choose(rng)
            .cloned()
            .expect("a pending seat has a move")
    }

    /// Only Hard searches: ~10 s in the browser, 48 simulations elsewhere.
    fn budget(level: Level, timed: bool) -> Option<Budget> {
        match level {
            Level::Hard if timed => Some(Budget::Millis(10_000)),
            Level::Hard => Some(Budget::Iterations(48)),
            Level::Easy | Level::Medium => None,
        }
    }

    fn own_search(
        s: &TwixtState,
        _seat: Seat,
        search: &Search,
        now_ms: &dyn Fn() -> f64,
    ) -> Option<TwixtAction> {
        let net = net()?;
        let mut rng = SmallRng::seed_from_u64(search.seed);
        match s.moves.as_slice() {
            [] => return Some(TwixtAction::Place(first_move(&mut rng))),
            [TwixtAction::Place(first)] if swap_wanted(*first) => return Some(TwixtAction::Swap),
            _ => {}
        }
        match search.level {
            Level::Easy => {
                let moves = policy(net, &s.board);
                moves
                    .choose_weighted(&mut rng, |&(_, prob)| prob)
                    .ok()
                    .map(|&(p, _)| TwixtAction::Place(p))
            }
            Level::Medium => best_policy_move(net, s),
            Level::Hard => {
                let budget = search.budget.unwrap_or(Budget::Iterations(48));
                Some(mcts(net, s, budget, now_ms).best)
            }
        }
    }
}
