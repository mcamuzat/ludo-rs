use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use super::{Action, Rocket, State};

impl State {
    /// What a hit is expected to cost `seat` now, from what everyone
    /// sees: the chance of each pairing rank times its value. For the AI:
    /// never in a view.
    pub fn hit_cost(&self, seat: Seat) -> f64 {
        let pool = self.supply.draw_pool();
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return f64::INFINITY;
        }
        let weighted: u32 = self.stacks[seat.index()]
            .iter()
            .map(|c| u32::from(pool[c.index()]) * u32::from(c.value()))
            .sum();
        f64::from(weighted) / f64::from(total)
    }
}

impl Heuristics for Rocket {
    /// Fold when a hit is expected to cost more than folding does.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, _rng: &mut R) -> Action {
        if !Self::legal_actions(s, seat).contains(&Action::Hit) {
            return Action::Fold;
        }
        if s.hit_cost(seat) > f64::from(s.fold_cost(seat)) {
            Action::Fold
        } else {
            Action::Hit
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::rocket::testing::table;
    use crate::rocket::{Action, Config, Rocket};

    #[test]
    fn folds_a_costly_stack_cheaply() {
        let mut rng = SmallRng::seed_from_u64(1);
        // A 1 in sight: folding costs 1; a hit risks the 10s and 9s.
        let s = table(&[&[10, 9, 8]], &[1], 3, &[10, 9, 8, 2]);
        assert_eq!(Rocket::rollout_action(&s, Seat(0), &mut rng), Action::Fold);
        let s = table(&[&[2]], &[9], 3, &[10, 9, 8, 7]);
        assert_eq!(Rocket::rollout_action(&s, Seat(0), &mut rng), Action::Hit);
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 1..=6u8 {
            for _ in 0..5 {
                let mut s = Rocket::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = Rocket::pending(&s).0.first() {
                    let action = Rocket::rollout_action(&s, seat, &mut rng);
                    assert!(Rocket::legal_actions(&s, seat).contains(&action));
                    Rocket::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
