use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use super::{Action, Blackstone, State, BLACK};

impl State {
    /// The chance, from what everyone sees, that taking `count` cards
    /// brings `seat` its second black stone. For the AI: never in a view.
    pub fn lose_chance(&self, seat: Seat, count: u8) -> f64 {
        let pool = self.supply.draw_pool();
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        let black = u32::from(pool[usize::from(BLACK - 1)]);
        let white = total.saturating_sub(black);
        let held = self.blacks[seat.index()];
        let count = u32::from(count).min(total);
        // Hypergeometric: the chance of fewer than `need` blacks in `count`.
        let need = u32::from(2u8.saturating_sub(held));
        let choose = |n: u32, k: u32| -> f64 {
            if k > n {
                return 0.0;
            }
            (0..k).fold(1.0, |acc, i| acc * f64::from(n - i) / f64::from(i + 1))
        };
        let all = choose(total, count);
        if all == 0.0 {
            return 0.0;
        }
        let safe: f64 = (0..need)
            .map(|b| choose(black, b) * choose(white, count.saturating_sub(b)))
            .sum();
        (1.0 - safe / all).clamp(0.0, 1.0)
    }
}

impl Heuristics for Blackstone {
    /// Take the fewest cards allowed unless that is likely to lose, then
    /// fold.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, _rng: &mut R) -> Action {
        let take = Action::Take { count: s.min_take };
        if !Self::legal_actions(s, seat).contains(&take) {
            return Action::Fold;
        }
        if s.lose_chance(seat, s.min_take) > 0.5 {
            Action::Fold
        } else {
            take
        }
    }

    /// Taking more than three over the minimum is never worth a search.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        Self::legal_actions(s, seat)
            .into_iter()
            .filter(|a| match a {
                Action::Take { count } => *count <= s.min_take.saturating_add(3),
                Action::Fold => true,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::blackstone::testing::table;
    use crate::blackstone::{Action, Blackstone, Config};

    #[test]
    fn the_chance_of_a_second_black_stone() {
        // One black among two cards, holding one already: take 1 → ½.
        let s = table(&[1, 0], &[], &[10, 3]);
        assert!((s.lose_chance(Seat(0), 1) - 0.5).abs() < 1e-9);
        assert!((s.lose_chance(Seat(0), 2) - 1.0).abs() < 1e-9);
        // Without one, a single black can never lose.
        assert_eq!(s.lose_chance(Seat(1), 2), 0.0);
        let mut rng = SmallRng::seed_from_u64(1);
        let s = table(&[1, 0], &[], &[10, 10, 3]);
        assert_eq!(
            Blackstone::rollout_action(&s, Seat(0), &mut rng),
            Action::Fold
        );
    }

    #[test]
    fn the_rollout_and_search_actions_are_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=8u8 {
            for _ in 0..5 {
                let mut s = Blackstone::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = Blackstone::pending(&s).0.first() {
                    let legal = Blackstone::legal_actions(&s, seat);
                    assert!(Blackstone::search_actions(&s, seat)
                        .iter()
                        .all(|a| legal.contains(a)));
                    let action = Blackstone::rollout_action(&s, seat, &mut rng);
                    assert!(legal.contains(&action));
                    Blackstone::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
