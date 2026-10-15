use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use super::{Action, Starboard, State, Status};

impl State {
    /// The chance, from what everyone sees, that the next card pairs the
    /// middle. For the AI: never in a view.
    pub fn pair_risk(&self) -> f64 {
        let pool = self.supply.draw_pool();
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        let hits: u32 = self.center.iter().map(|c| u32::from(pool[c.index()])).sum();
        if total == 0 {
            1.0
        } else {
            f64::from(hits) / f64::from(total)
        }
    }
}

impl Heuristics for Starboard {
    /// Lock when a pair is likely, hit otherwise; with locked players to
    /// knock out, hit a little longer.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, _rng: &mut R) -> Action {
        if !Self::legal_actions(s, seat).contains(&Action::Hit) {
            return Action::Lock;
        }
        let locked = s.status.contains(&Status::Locked);
        let threshold = if locked { 0.7 } else { 0.55 };
        if s.pair_risk() >= threshold {
            Action::Lock
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

    use crate::starboard::testing::table;
    use crate::starboard::{Action, Config, Starboard};

    #[test]
    fn locks_a_full_middle_and_hits_a_thin_one() {
        let mut rng = SmallRng::seed_from_u64(1);
        let s = table(3, 6, &[10, 9, 8, 7, 6, 5], &[10, 9, 8, 7, 1]);
        assert_eq!(
            Starboard::rollout_action(&s, Seat(0), &mut rng),
            Action::Lock
        );
        let s = table(3, 6, &[1, 2], &[10, 9, 8, 7, 3]);
        assert_eq!(
            Starboard::rollout_action(&s, Seat(0), &mut rng),
            Action::Hit
        );
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=7u8 {
            for _ in 0..5 {
                let mut s = Starboard::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = Starboard::pending(&s).0.first() {
                    let action = Starboard::rollout_action(&s, seat, &mut rng);
                    assert!(Starboard::legal_actions(&s, seat).contains(&action));
                    Starboard::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
