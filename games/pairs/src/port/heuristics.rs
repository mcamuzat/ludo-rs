use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use super::{Action, Port, State, Status};

impl State {
    /// The chance, from what everyone sees, that the next card pairs
    /// `seat`'s stack. For the AI: never in a view.
    pub fn pair_risk(&self, seat: Seat) -> f64 {
        let pool = self.supply.draw_pool();
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 1.0;
        }
        let hits: u32 = self.stacks[seat.index()]
            .iter()
            .map(|c| u32::from(pool[c.index()]))
            .sum();
        f64::from(hits) / f64::from(total)
    }
}

impl Heuristics for Port {
    /// Lock once ahead of every locked stack and the risk is real, or
    /// once the risk is high; otherwise hit.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, _rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if !legal.contains(&Action::Hit) {
            return Action::Lock;
        }
        let risk = s.pair_risk(seat);
        let best_locked = (0..s.players)
            .map(Seat)
            .filter(|&o| o != seat && s.status[o.index()] == Status::Locked)
            .map(|o| s.total(o))
            .max()
            .unwrap_or(0);
        let ahead = s.total(seat) > best_locked;
        if risk >= 0.5 || (ahead && risk >= 0.25) {
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

    use crate::port::testing::table;
    use crate::port::{Action, Config, Port};

    #[test]
    fn locks_a_risky_stack_and_hits_a_safe_one() {
        let s = table(&[&[9, 8, 7], &[1]], &[9, 8, 7, 2]);
        let mut rng = SmallRng::seed_from_u64(1);
        assert_eq!(Port::rollout_action(&s, Seat(0), &mut rng), Action::Lock);
        let s = table(&[&[1], &[2]], &[9, 8, 7, 3]);
        assert_eq!(Port::rollout_action(&s, Seat(0), &mut rng), Action::Hit);
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=7u8 {
            for _ in 0..5 {
                let mut s = Port::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = Port::pending(&s).0.first() {
                    let action = Port::rollout_action(&s, seat, &mut rng);
                    assert!(Port::legal_actions(&s, seat).contains(&action));
                    Port::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
