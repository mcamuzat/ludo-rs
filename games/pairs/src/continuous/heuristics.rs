use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, ContinuousPairs, State};
use crate::deck::Rank;

/// Below this gap (in points) between folding and hitting, either is played.
const CLOSE: f64 = 0.5;

/// The folds on the lowest card in play, from whoever holds one.
fn cheapest_folds(s: &State) -> Vec<Action> {
    let Some(low) = s.lowest_in_play() else {
        return Vec::new();
    };
    s.in_play()
        .filter(|&(_, card)| card == low)
        .map(|(seat, card)| Action::Fold { seat, card })
        .collect()
}

impl Heuristics for ContinuousPairs {
    /// Fold when the sure cost of the lowest card is below the points a hit
    /// puts at risk; never fold into the target while a card can be drawn.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let Some(fold) = cheapest_folds(s).choose(rng).cloned() else {
            return Action::Hit;
        };
        if !s.supply.can_draw() {
            return fold;
        }
        let Action::Fold { card, .. } = fold else {
            return Action::Hit;
        };
        if s.score(seat) + card.value() >= s.target {
            return Action::Hit;
        }
        let margin = s.expected_hit_cost(seat) - f64::from(card.value());
        if margin > CLOSE || (margin > -CLOSE && rng.gen_bool(0.5)) {
            fold
        } else {
            Action::Hit
        }
    }

    /// A hit, and the folds on the two lowest ranks in play: paying more than
    /// that to fold is never worth searching. Stacks are public, so this
    /// depends only on what `seat` sees.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let mut in_play: Vec<Rank> = s.in_play().map(|(_, card)| card).collect();
        in_play.sort_unstable();
        in_play.dedup();
        let legal = Self::legal_actions(s, seat);
        let Some(&limit) = in_play.get(1).or(in_play.first()) else {
            return legal;
        };
        legal
            .into_iter()
            .filter(|action| match action {
                Action::Hit => true,
                Action::Fold { card, .. } => *card <= limit,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::continuous::testing::{r, table, table_with_rest};
    use crate::continuous::{Action, ContinuousPairs};

    fn rollout(s: &crate::continuous::State, seed: u64) -> Action {
        ContinuousPairs::rollout_action(s, Seat(0), &mut SmallRng::seed_from_u64(seed))
    }

    #[test]
    fn folds_cheaply_rather_than_risk_a_big_stack() {
        // About 4.3 points at risk on a hit; folding on the 1 costs 1.
        let s = table_with_rest(&[&[10, 9, 8], &[1]]);
        for seed in 0..30 {
            assert_eq!(
                rollout(&s, seed),
                Action::Fold {
                    seat: Seat(1),
                    card: r(1)
                }
            );
        }
    }

    #[test]
    fn hits_when_folding_costs_more_than_the_risk() {
        // About 1.7 points at risk; the cheapest fold costs 9.
        let s = table_with_rest(&[&[10], &[9]]);
        for seed in 0..30 {
            assert_eq!(rollout(&s, seed), Action::Hit);
        }
    }

    #[test]
    fn never_folds_into_the_target() {
        let mut s = table_with_rest(&[&[10, 9, 8], &[2]]);
        s.scored[0] = vec![r(10), r(10), r(9)];
        assert_eq!(s.score(Seat(0)) + 2, s.target);
        for seed in 0..30 {
            assert_eq!(rollout(&s, seed), Action::Hit);
        }
    }

    #[test]
    fn folds_when_there_is_nothing_to_draw() {
        let s = table(&[&[10], &[9]], &[]);
        for seed in 0..30 {
            assert_eq!(
                rollout(&s, seed),
                Action::Fold {
                    seat: Seat(1),
                    card: r(9)
                }
            );
        }
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        use crate::continuous::Config;
        use rand::Rng;
        let mut rng = SmallRng::seed_from_u64(11);
        for players in 2..=8u8 {
            let mut s = ContinuousPairs::new(&Config::default(), players, rng.gen());
            while let Some(&seat) = ContinuousPairs::pending(&s).0.first() {
                let action = ContinuousPairs::rollout_action(&s, seat, &mut rng);
                assert!(ContinuousPairs::legal_actions(&s, seat).contains(&action));
                ContinuousPairs::apply(&mut s, seat, &action).unwrap();
            }
        }
    }

    #[test]
    fn the_search_only_weighs_the_two_lowest_ranks() {
        let s = table_with_rest(&[&[10, 7], &[3, 9], &[3, 5]]);
        let actions = ContinuousPairs::search_actions(&s, Seat(0));
        let fold = |seat: u8, card: u8| Action::Fold {
            seat: Seat(seat),
            card: r(card),
        };
        assert_eq!(actions.len(), 4);
        for wanted in [Action::Hit, fold(1, 3), fold(2, 3), fold(2, 5)] {
            assert!(actions.contains(&wanted), "{wanted:?}");
        }
        let legal = ContinuousPairs::legal_actions(&s, Seat(0));
        assert!(actions.iter().all(|a| legal.contains(a)));
    }

    #[test]
    fn the_search_keeps_a_fold_when_it_is_the_only_move() {
        let s = table(&[&[10], &[9]], &[]);
        assert!(!ContinuousPairs::search_actions(&s, Seat(0)).is_empty());
    }
}
