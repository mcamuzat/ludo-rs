//! Easy and Medium play a policy; Hard searches the best-scoring placements.

use ludo_core::ai::Level;
use ludo_core::{Heuristics, Seat};
use rand::Rng;

use crate::{is_double, Action, State, WuHsing};

/// How many placements Hard searches.
const SEARCH_WIDTH: usize = 12;
/// Rounds a simulation plays before its standing is estimated.
const HORIZON: usize = 2;
/// Score gap worth about 73 % against one opponent.
const SCALE: f64 = 30.0;

/// Provisional final score: the score less the hand still held.
fn standing(s: &State, seat: usize) -> f64 {
    let held: i32 = s.hands[seat]
        .iter()
        .map(|&d| if is_double(d) { 5 } else { 25 })
        .sum();
    f64::from(s.scores[seat] - held)
}

/// The legal actions of `seat`, best-scoring first (stable).
fn ranked(s: &State, seat: Seat) -> Vec<(Action, u32)> {
    let mut moves = s.moves(seat);
    moves.sort_by_key(|&(_, points)| std::cmp::Reverse(points));
    moves
}

fn pick<R: Rng + ?Sized>(moves: &[(Action, u32)], rng: &mut R) -> Action {
    moves[(rng.gen::<u32>() % moves.len() as u32) as usize].0
}

impl Heuristics for WuHsing {
    /// The best-scoring placement, ties drawn at random.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let moves = ranked(s, seat);
        let best = moves[0].1;
        let top = moves.iter().take_while(|&&(_, p)| p == best).count();
        pick(&moves[..top], rng)
    }

    /// After two rounds, each seat's share of opponents it leads, softened
    /// by the score gap.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        let n = sim.hands.len();
        if sim.finish.is_some()
            || sim.board.placed().len() < root.board.placed().len() + HORIZON * n
        {
            return None;
        }
        let v: Vec<f64> = (0..n).map(|i| standing(sim, i)).collect();
        Some(
            (0..n)
                .map(|i| {
                    let sum: f64 = (0..n)
                        .filter(|&j| j != i)
                        .map(|j| 1.0 / (1.0 + (-(v[i] - v[j]) / SCALE).exp()))
                        .sum();
                    sum / (n - 1) as f64
                })
                .collect(),
        )
    }

    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        ranked(s, seat)
            .into_iter()
            .take(SEARCH_WIDTH)
            .map(|(a, _)| a)
            .collect()
    }

    /// Easy: one of the 5 best-scoring placements. Medium: the best one.
    fn policy_action<R: Rng + ?Sized>(
        s: &State,
        seat: Seat,
        level: Level,
        rng: &mut R,
    ) -> Option<Action> {
        let moves = ranked(s, seat);
        match level {
            Level::Easy => Some(pick(&moves[..moves.len().min(5)], rng)),
            Level::Medium => Some(moves[0].0),
            Level::Hard => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::ai::Level;
    use ludo_core::{Game, Heuristics};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::{Config, WuHsing};

    #[test]
    fn evaluate_cuts_after_two_rounds() {
        let root = WuHsing::new(&Config::default(), 3, 4);
        assert!(WuHsing::evaluate(&root, &root).is_none());
        let mut sim = root.clone();
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        while sim.board.placed().len() < root.board.placed().len() + 6 {
            let seat = sim.turn.unwrap();
            let a = WuHsing::rollout_action(&sim, seat, &mut rng);
            WuHsing::apply(&mut sim, seat, &a).unwrap();
        }
        sim.scores = vec![300, 0, 0];
        let v = WuHsing::evaluate(&root, &sim).expect("two rounds played");
        assert_eq!(v.len(), 3);
        assert!(v.iter().all(|x| (0.0..=1.0).contains(x)));
        assert!(v[0] > 0.9 && v[1] < 0.5);
    }

    #[test]
    fn levels_pick_legal_moves() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        for players in [2u8, 4] {
            for seed in 0..10 {
                let mut s = WuHsing::new(&Config::default(), players, seed);
                let mut k = 0u32;
                while let Some(seat) = s.turn {
                    let moves = s.moves(seat);
                    let best = moves.iter().map(|&(_, p)| p).max().unwrap();
                    let legal = WuHsing::legal_actions(&s, seat);
                    let medium = WuHsing::policy_action(&s, seat, Level::Medium, &mut rng).unwrap();
                    assert!(moves.contains(&(medium, best)));
                    let easy = WuHsing::policy_action(&s, seat, Level::Easy, &mut rng).unwrap();
                    assert!(legal.contains(&easy));
                    assert!(WuHsing::policy_action(&s, seat, Level::Hard, &mut rng).is_none());
                    let search = WuHsing::search_actions(&s, seat);
                    assert!(!search.is_empty() && search.len() <= 12);
                    assert!(search.iter().all(|a| legal.contains(a)));
                    let roll = WuHsing::rollout_action(&s, seat, &mut rng);
                    assert!(moves.contains(&(roll, best)));
                    let next = [easy, medium, roll][(k % 3) as usize];
                    k += 1;
                    WuHsing::apply(&mut s, seat, &next).unwrap();
                }
            }
        }
    }
}
